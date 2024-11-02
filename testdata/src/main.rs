use actix_web::{web, App, HttpResponse, HttpServer};
use actix_cors::Cors;
use serde::{Serialize, Deserialize};
use rand::Rng;
use std::time::Duration;
use tokio::time::interval;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ChartData {
    timestamp: i64,
    values: Vec<f64>,
    labels: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TableData {
    id: i32,
    name: String,
    value: f64,
    status: String
}

struct AppState {
    chart_data: Arc<Mutex<ChartData>>,
    table_data: Arc<Mutex<Vec<TableData>>>
}

async fn get_chart_data(data: web::Data<AppState>) -> HttpResponse {
    let chart_data = data.chart_data.lock().unwrap();
    let response = HttpResponse::Ok().json(&*chart_data);
    
    println!("GET /api/chart - Response: {:?}", &*chart_data);
    response
}

async fn get_table_data(data: web::Data<AppState>) -> HttpResponse {
    let table_data = data.table_data.lock().unwrap();
    let response = HttpResponse::Ok().json(&*table_data);
    
    println!("GET /api/table - Response: {:?}", &*table_data);
    response
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let chart_data = Arc::new(Mutex::new(ChartData {
        timestamp: 0,
        values: vec![],
        labels: vec![]
    }));

    let table_data = Arc::new(Mutex::new(Vec::new()));
    
    let chart_data_clone = chart_data.clone();
    let table_data_clone = table_data.clone();

    // Background task to update data every 5 seconds
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let mut rng = rand::thread_rng();
            
            // Update chart data
            let mut chart = chart_data_clone.lock().unwrap();
            *chart = ChartData {
                timestamp: chrono::Utc::now().timestamp(),
                values: (0..5).map(|_| rng.gen_range(-100.0..100.0)).collect(),
                labels: (0..5).map(|i| format!("Series {}", i)).collect()
            };

            // Update table data
            let mut table = table_data_clone.lock().unwrap();
            *table = (0..10).map(|i| TableData {
                id: i,
                name: format!("Item {}", i),
                value: rng.gen_range(0.0..1000.0),
                status: ["Active", "Pending", "Inactive"][rng.gen_range(0..3)].to_string()
            }).collect();
        }
    });

    let app_state = web::Data::new(AppState {
        chart_data,
        table_data
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(actix_web::middleware::Logger::default())
            .app_data(app_state.clone())
            .route("/api/chart", web::get().to(get_chart_data))
            .route("/api/table", web::get().to(get_table_data))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}