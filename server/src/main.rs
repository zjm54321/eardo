#![recursion_limit = "256"]

use app::*;
use axum::{Extension, Router};
use dotenv::dotenv;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use sqlx::sqlite::SqlitePoolOptions;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let db_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:eardo.db?mode=rwc".to_string());
    log!("连接数据库: {}", db_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to create pool.");

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            let pool = pool.clone();
            move || {
                provide_context(pool.clone());
                shell(leptos_options.clone())
            }
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(Extension(pool))
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
