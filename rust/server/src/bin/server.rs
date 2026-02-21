mod grpc_pb {
    tonic::include_proto!("teacher");
}

use std::fs::OpenOptions;

use clap::Parser;
use env_logger::{Builder, Target};
use log::info;
use grpc_pb::calculator_server::{Calculator, CalculatorServer};
use tonic::{transport::Server, Request, Response, Status};
use std::io::Write;

#[derive(Default)]
struct MyCalculator;

#[tonic::async_trait]
impl Calculator for MyCalculator {
    async fn add(
        &self,
        request: Request<grpc_pb::Numbers>,
    ) -> Result<Response<grpc_pb::Result>, Status> {
        let remote_addr = request.remote_addr().unwrap_or_else(|| "unknown".parse().unwrap()).to_string();
        let r = request.into_inner();
        info!("Adding [{}] {} and {}", remote_addr,r.num1, r.num2);
        let result = r.num1 as i64 + r.num2 as i64;
        info!("Result of addition [{}]: {}", remote_addr, result);
        Ok(Response::new(grpc_pb::Result { result: result }))
    }

    async fn sub(
        &self,
        request: Request<grpc_pb::Numbers>,
    ) -> Result<Response<grpc_pb::Result>, Status> {
        let remote_addr = request.remote_addr().unwrap_or_else(|| "unknown".parse().unwrap()).to_string();
        let r = request.into_inner();
        info!("Subtracting [{}] {} and {}", remote_addr, r.num1, r.num2);
        let result = r.num1 as i64 - r.num2 as i64;
        info!("Result of subtraction [{}]: {}", remote_addr, result);
        Ok(Response::new(grpc_pb::Result { result: result }))
    }

    async fn mul(
        &self,
        request: Request<grpc_pb::Numbers>,
    ) -> Result<Response<grpc_pb::Result>, Status> {
        let remote_addr = request.remote_addr().unwrap_or_else(|| "unknown".parse().unwrap()).to_string();
        let r = request.into_inner();
        info!("Multiplying [{}] {} and {}", remote_addr, r.num1, r.num2);
        let result = r.num1 as i64 * r.num2 as i64;
        info!("Result of multiplication [{}]: {}", remote_addr, result);
        Ok(Response::new(grpc_pb::Result { result: result }))
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// Math Teacher gRPC Server
struct Args {
    /// server host
    #[arg(long, default_value = "[::1]")]
    host: String,
    /// server port
    #[arg(long, default_value_t = 10000)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args= Args::parse();
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("server.log")
        .unwrap();
    Builder::new()
        .target(Target::Pipe(Box::new(file)))
        .filter_level(log::LevelFilter::Info)
        .format(|buf, record| {
            let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

            writeln!(buf, "{} - {} - {} - {}", ts, record.module_path().unwrap_or(record.target()), record.level(), record.args())
        })
        .init();

    info!("Starting server on {}:{}", args.host, args.port);

    let addr = format!("{}:{}", args.host, args.port).parse()?;

    let svc = CalculatorServer::new(MyCalculator::default());

    Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?;

    Ok(())
}