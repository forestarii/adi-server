// use crate::{model::UserModel, AppState};
// use actix_web::{web, HttpResponse};
// use base64;

// pub async fn basic_auth(
//     auth_header: &str,
//     db_pool: web::Data<AppState>,
// ) -> Result<UserModel, HttpResponse> {
//     if auth_header.starts_with("Basic ") {
//         let encoded_creds = auth_header.trim_start_matches("Basic ");
//         if let Ok(decoded) = base64::decode(encoded_creds) {
//             if let Ok(creds) = String::from_utf8(decoded) {
//                 let user_pass: Vec<&str> = creds.splitn(2, ':').collect();
//                 if user_pass.len() == 2 {
//                     let username = user_pass[0];
//                     let password = user_pass[1];

//                     // Fetch user from the database
//                     let result = sqlx::query_as::<_, UserModel>(
//                         "SELECT id, username, password FROM users WHERE username = ?",
//                     )
//                     .bind(username)
//                     .fetch_one(&db_pool.db)
//                     .await;

//                     match result {
//                         Ok(user) => {
//                             // Check if password matches
//                             if user.password == password {
//                                 return Ok(user);
//                             }
//                         }
//                         Err(_) => (),
//                     }
//                 }
//             }
//         }
//     }
//     Err(HttpResponse::Unauthorized().finish())
// }
