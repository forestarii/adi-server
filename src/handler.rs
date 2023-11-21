use crate::{
    model::{GeneratedValueModel, GeneratedValueModelResponse},
    schema::{CreateGeneratedValueSchema, CreateUserSchema, UpdateGeneratedValueSchema},
    AppState,
};
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use rand::{thread_rng, Rng};
use reqwest;
use serde_json::{json, Value};
use sqlx;
use std::sync::{Arc, Mutex};
use tokio;
use uuid::Uuid;

#[get("/healthchecker")]
async fn health_checker_handler() -> impl Responder {
    const MESSAGE: &str = "adi-server still alive";
    HttpResponse::Ok().json(json!({"status": "success","message": MESSAGE}))
}

#[post("/run")]
async fn execute_value_handler(pool: web::Data<AppState>) -> impl Responder {
    const URL: &str = "https://httpbin.org/post";
    let values: Arc<Mutex<Vec<i8>>> = Arc::new(Mutex::new(Vec::new()));

    let tasks = (0..30).map(|_| {
        let values = Arc::clone(&values);
        let url = URL.to_string();
        let pool = pool.clone();

        tokio::spawn(async move {
            let rand_value: i8 = thread_rng().gen_range(0..10);
            let request_json = json!({ "value": rand_value });

            if let Ok(result) = reqwest::Client::new()
                .post(&url)
                .json(&request_json)
                .send()
                .await
            {
                if let Ok(text) = result.text().await {
                    if let Ok(v) = serde_json::from_str::<Value>(&text) {
                        if let Some(json_value) = v.get("json") {
                            if let Some(value) = json_value.get("value") {
                                if let Some(i8_val) = value.as_i64().map(|v| v as i8) {
                                    values.lock().unwrap().push(i8_val);

                                    let generated_value =
                                        CreateGeneratedValueSchema { num: Some(i8_val) };
                                    insert_generated_value(pool, generated_value).await;
                                }
                            }
                        }
                    }
                }
            }
        })
    });

    for task in tasks {
        let _ = task.await;
    }

    let locked_values = values.lock().unwrap();
    let cloned_vector = locked_values.clone();

    let sorted_vector = find_repeated_values(&cloned_vector);

    HttpResponse::Ok()
        .json(json!({"status": "success", "output values": cloned_vector, "repeted values": sorted_vector}))
}

fn find_repeated_values(values: &[i8]) -> Vec<i8> {
    let mut repeated_values = values
        .iter()
        .filter(|&&x| values.iter().filter(|&&y| x == y).count() > 1)
        .cloned()
        .collect::<Vec<i8>>();

    repeated_values.sort();
    repeated_values.dedup();

    repeated_values
}

async fn insert_generated_value(pool: web::Data<AppState>, body: CreateGeneratedValueSchema) {
    let id = Uuid::new_v4().to_string();
    let _query_result = sqlx::query(r#"INSERT INTO generated_values (id, num) VALUES (?, ?)"#)
        .bind(id.clone())
        .bind(body.num.to_owned())
        .execute(&pool.db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());
}

#[post("/generated-value/")]
async fn create_generated_value_handler(
    pool: web::Data<AppState>,
    body: web::Json<CreateGeneratedValueSchema>,
) -> impl Responder {
    let id = Uuid::new_v4().to_string();
    let query_result = sqlx::query(r#"INSERT INTO generated_values (id, num) VALUES (?, ?)"#)
        .bind(id.clone())
        .bind(body.num.to_owned())
        .execute(&pool.db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());

    if let Err(err) = query_result {
        if err.contains("Duplicate entry") {
            return HttpResponse::BadRequest().json(
            serde_json::json!({"status": "fail","message": "Generated value with that id already exists"}),
        );
        }
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"status": "error","message": format!("{:?}", err)}));
    }

    let query_result = sqlx::query_as!(
        GeneratedValueModel,
        r#"SELECT * FROM generated_values WHERE id = ?"#,
        id
    )
    .fetch_one(&pool.db)
    .await;

    match query_result {
        Ok(generated_value) => {
            let generated_value_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "generated_value": filter_db_record(&generated_value)
            })});

            return HttpResponse::Ok().json(generated_value_response);
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
        }
    }
}

#[get("/generated-values")]
pub async fn generated_values_handler(pool: web::Data<AppState>) -> impl Responder {
    let generated_values: Vec<GeneratedValueModel> =
        sqlx::query_as!(GeneratedValueModel, r#"SELECT * FROM generated_values"#)
            .fetch_all(&pool.db)
            .await
            .unwrap();

    let generated_values_response = generated_values
        .into_iter()
        .map(|generated_value| filter_db_record(&generated_value))
        .collect::<Vec<GeneratedValueModelResponse>>();

    let json_response = serde_json::json!({
        "status": "success",
        "results": generated_values_response.len(),
        "generated_values": generated_values_response
    });
    HttpResponse::Ok().json(json_response)
}

#[get("/generated-value/{id}")]
async fn get_generated_value_handler(
    path: web::Path<uuid::Uuid>,
    pool: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner().to_string();
    let query_result = sqlx::query_as!(
        GeneratedValueModel,
        r#"SELECT * FROM generated_values WHERE id = ?"#,
        id
    )
    .fetch_one(&pool.db)
    .await;

    match query_result {
        Ok(generated_value) => {
            let generated_value_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "generated_value": filter_db_record(&generated_value)
            })});

            return HttpResponse::Ok().json(generated_value_response);
        }
        Err(sqlx::Error::RowNotFound) => {
            return HttpResponse::NotFound().json(
            serde_json::json!({"status": "fail","message": format!("Generated Value with ID: {} not found", id)}),
        );
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
        }
    };
}

#[patch("/generated-value/{id}")]
async fn edit_generated_value_handler(
    path: web::Path<uuid::Uuid>,
    body: web::Json<UpdateGeneratedValueSchema>,
    pool: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner().to_string();
    let query_result = sqlx::query_as!(
        GeneratedValueModel,
        r#"SELECT * FROM generated_values WHERE id = ?"#,
        id
    )
    .fetch_one(&pool.db)
    .await;

    let _generated_value = match query_result {
        Ok(value) => value,
        Err(sqlx::Error::RowNotFound) => {
            return HttpResponse::NotFound().json(
                serde_json::json!({"status": "fail","message": format!("Generated Value with ID: {} not found", id)}),
            );
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
        }
    };

    let update_result = sqlx::query(r#"UPDATE generated_values SET num = ? WHERE id = ?"#)
        .bind(body.num.to_owned().unwrap())
        .bind(id.to_owned())
        .execute(&pool.db)
        .await;

    match update_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                let message = format!("Generated value with ID: {} not found", id);
                return HttpResponse::NotFound().json(json!({"status": "fail","message": message}));
            }
        }
        Err(e) => {
            let message = format!("Internal server error: {}", e);
            return HttpResponse::InternalServerError()
                .json(json!({"status": "error","message": message}));
        }
    }

    let updated_generated_value_result = sqlx::query_as!(
        GeneratedValueModel,
        r#"SELECT * FROM generated_values WHERE id = ?"#,
        id.to_owned()
    )
    .fetch_one(&pool.db)
    .await;

    match updated_generated_value_result {
        Ok(generated_value) => {
            let generated_value_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "Generated Value": filter_db_record(&generated_value)
            })});

            HttpResponse::Ok().json(generated_value_response)
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"status": "error","message": format!("{:?}", e)})),
    }
}

#[delete("/generated-value/{id}")]
async fn delete_generated_value_handler(
    path: web::Path<uuid::Uuid>,
    pool: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner().to_string();
    let query_result = sqlx::query!(r#"DELETE FROM generated_values WHERE id = ?"#, id)
        .execute(&pool.db)
        .await;

    match query_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                let message = format!("Generated Value with ID: {} not found", id);
                HttpResponse::NotFound().json(json!({"status": "fail","message": message}))
            } else {
                HttpResponse::NoContent().finish()
            }
        }
        Err(e) => {
            let message = format!("Internal server error: {}", e);
            HttpResponse::InternalServerError().json(json!({"status": "error","message": message}))
        }
    }
}

#[delete("/remove-generated-values")]
async fn remove_all_generated_values_handler(pool: web::Data<AppState>) -> impl Responder {
    let _ = sqlx::query("DELETE FROM generated_values")
        .execute(&pool.db)
        .await;

    HttpResponse::NoContent().finish()
}

#[post("/create-user")]
pub async fn store_user_handler(
    pool: web::Data<AppState>,
    user: web::Json<CreateUserSchema>,
) -> impl Responder {
    let id = Uuid::new_v4().to_string();

    let _ = sqlx::query("INSERT INTO users (id, username, user_password) VALUES (?, ?, ?)")
        .bind(id.clone())
        .bind(user.username.clone())
        .bind(user.password.clone())
        .execute(&pool.db)
        .await
        .map(|_| ());

    HttpResponse::Ok().json(json!({"status": "success"}))
}

#[get("/user/{id}")]
async fn get_user_handler(
    path: web::Path<uuid::Uuid>,
    pool: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner().to_string();
    let _query_result = sqlx::query!(r#"SELECT * FROM users WHERE id = ?"#, id)
        .fetch_one(&pool.db)
        .await;

    HttpResponse::Ok().json(json!({"value": 1}))
}

fn filter_db_record(generated_values: &GeneratedValueModel) -> GeneratedValueModelResponse {
    GeneratedValueModelResponse {
        id: generated_values.id.to_owned(),
        num: generated_values.num.unwrap(),
    }
}

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(health_checker_handler)
        .service(execute_value_handler)
        .service(create_generated_value_handler)
        .service(generated_values_handler)
        .service(get_generated_value_handler)
        .service(edit_generated_value_handler)
        .service(delete_generated_value_handler)
        .service(remove_all_generated_values_handler);

    conf.service(scope);
}
