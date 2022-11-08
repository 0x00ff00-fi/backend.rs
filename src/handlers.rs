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
