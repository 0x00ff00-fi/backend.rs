mod config {
    use serde::Deserialize;
    #[derive(Debug, Default, Deserialize)]
    pub struct ExampleConfig {
        pub server_addr: String,
        pub pg: deadpool_postgres::Config,
    }
}

mod models {
    use chrono::NaiveDateTime;
    use serde::{Deserialize, Serialize};
    use tokio_pg_mapper_derive::PostgresMapper;

    #[derive(Deserialize, PostgresMapper, Serialize)]
    #[pg_mapper(table = "users")] // singular 'user' is a keyword..
    pub struct User {
        pub email: String,
        pub first_name: String,
        pub last_name: String,
        pub username: String,
    }

    #[derive(Deserialize, PostgresMapper, Serialize)]
    #[pg_mapper(table = "posts")] // singular 'user' is a keyword..
    pub struct Post {
        pub name: String,
        pub icon: String,
        pub content: String,
        pub media: Option<String>,
        pub created_at: Option<NaiveDateTime>,
    }
}

mod errors {
    use actix_web::{HttpResponse, ResponseError};
    use deadpool_postgres::PoolError;
    use derive_more::{Display, From};
    use tokio_pg_mapper::Error as PGMError;
    use tokio_postgres::error::Error as PGError;

    #[derive(Display, From, Debug)]
    pub enum MyError {
        NotFound,
        PGError(PGError),
        PGMError(PGMError),
        PoolError(PoolError),
    }
    impl std::error::Error for MyError {}

    impl ResponseError for MyError {
        fn error_response(&self) -> HttpResponse {
            match *self {
                MyError::NotFound => HttpResponse::NotFound().finish(),
                MyError::PoolError(ref err) => {
                    HttpResponse::InternalServerError().body(err.to_string())
                }
                _ => HttpResponse::InternalServerError().finish(),
            }
        }
    }
}

mod db {
    use deadpool_postgres::Client;
    use tokio_pg_mapper::FromTokioPostgresRow;

    use crate::{
        errors::MyError,
        models::{Post, User},
    };

    pub async fn add_user(client: &Client, user_info: User) -> Result<User, MyError> {
        let _stmt = include_str!("../sql/add_user.sql");
        let _stmt = _stmt.replace("$table_fields", &User::sql_table_fields());
        let stmt = client.prepare_cached(&_stmt).await.unwrap();

        client
            .query(
                &stmt,
                &[
                    &user_info.email,
                    &user_info.first_name,
                    &user_info.last_name,
                    &user_info.username,
                ],
            )
            .await?
            .iter()
            .map(|row| User::from_row_ref(row).unwrap())
            .collect::<Vec<User>>()
            .pop()
            .ok_or(MyError::NotFound) // more applicable for SELECTs
    }

    pub async fn add_post(client: &Client, post_info: Post) -> Result<Post, MyError> {
        let _stmt = include_str!("../sql/add_post.sql");
        let _stmt = _stmt.replace("$table_fields", &Post::sql_table_fields());
        let stmt = client.prepare_cached(&_stmt).await.unwrap();

        client
            .query(
                &stmt,
                &[
                    &post_info.name,
                    &post_info.icon,
                    &post_info.content,
                    &post_info.media,
                ],
            )
            .await?
            .iter()
            .map(|row| Post::from_row_ref(row).unwrap())
            .collect::<Vec<Post>>()
            .pop()
            .ok_or(MyError::NotFound) // more applicable for SELECTs
    }

    pub async fn get_users(client: &Client) -> Result<Vec<User>, MyError> {
        let stmt = client.prepare_cached("SELECT * FROM users").await.unwrap();

        Ok(client
            .query(&stmt, &[])
            .await?
            .iter()
            .map(|row| User::from_row_ref(row).unwrap())
            .collect::<Vec<User>>())
    }

    pub async fn get_posts(client: &Client) -> Result<Vec<Post>, MyError> {
        let stmt = client
            .prepare_cached("SELECT * FROM posts ORDER BY created_at DESC LIMIT 100")
            .await
            .unwrap();

        Ok(client
            .query(&stmt, &[])
            .await?
            .iter()
            .map(|row| Post::from_row_ref(row).unwrap())
            .collect::<Vec<Post>>())
    }
}

mod handlers {
    use actix_web::{web, Error, HttpResponse};
    use deadpool_postgres::{Client, Pool};

    use crate::{
        db,
        errors::MyError,
        models::{Post, User},
    };

    pub async fn add_user(
        user: web::Json<User>,
        db_pool: web::Data<Pool>,
    ) -> Result<HttpResponse, Error> {
        let user_info: User = user.into_inner();
        let client: Client = db_pool.get().await.map_err(MyError::PoolError)?;
        let new_user = db::add_user(&client, user_info).await?;
        Ok(HttpResponse::Ok().json(new_user))
    }

    pub async fn add_post(
        post: web::Json<Post>,
        db_pool: web::Data<Pool>,
    ) -> Result<HttpResponse, Error> {
        let post_info: Post = post.into_inner();
        let client: Client = db_pool.get().await.map_err(MyError::PoolError)?;
        let new_post = db::add_post(&client, post_info).await?;
        Ok(HttpResponse::Ok().json(new_post))
    }

    pub async fn get_users(db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
        let client: Client = db_pool.get().await.map_err(MyError::PoolError)?;
        let users = db::get_users(&client).await?;
        Ok(HttpResponse::Ok().json(users))
    }

    pub async fn get_posts(db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
        let client: Client = db_pool.get().await.map_err(MyError::PoolError)?;
        let posts = db::get_posts(&client).await?;
        Ok(HttpResponse::Ok().json(posts))
    }

    pub async fn root() -> Result<HttpResponse, Error> {
        Ok(HttpResponse::Ok().body("test"))
    }
}

use ::config::Config;
use actix_web::{middleware, web, App, HttpServer};
use dotenv::dotenv;
use handlers::{add_post, add_user, get_posts, get_users, root};
use tokio_postgres::NoTls;

use crate::config::ExampleConfig;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config_ = Config::builder()
        .add_source(::config::Environment::default())
        .build()
        .unwrap();

    let config: ExampleConfig = config_.try_deserialize().unwrap();
    let pool = config.pg.create_pool(None, NoTls).unwrap();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .service(web::resource("/").route(web::get().to(root)))
            .service(
                web::resource("/users")
                    .route(web::post().to(add_user))
                    .route(web::get().to(get_users)),
            )
            .service(
                web::resource("/posts")
                    .route(web::post().to(add_post))
                    .route(web::get().to(get_posts)),
            )
    })
    .bind(config.server_addr.clone())?
    .run();
    println!("Server running at http://{}/", config.server_addr);

    server.await
}
