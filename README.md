# How to run the project

1. Download or Clone repository by HTTPS/SSH/GitHub CLI

2. Create and run MySQL server in the docker container by. Also, check if docker engine already running:
   **docker-compose up -d**

3. Download sqlx-cli tools for running migrations and by:
   a) install tools
   **cargo install sqlx-cli**

   b) create table from migrations
   **sqlx migrate run**

4. Build and Run project
   **cargo build**
