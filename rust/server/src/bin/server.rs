mod grpc_pb {
    #![allow(clippy::all)]
    #![allow(warnings)]
    tonic::include_proto!("teacher");
}

use std::{fs::OpenOptions, io, sync::Mutex};

use clap::Parser;
use env_logger::{Builder, Target};
use grpc_pb::calculator_server::{Calculator, CalculatorServer};
use log::info;
use std::io::Write;
use tonic::{Request, Response, Status, transport::Server};

struct MultiWriter {
    file: Mutex<std::fs::File>,
}

impl Write for MultiWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file
            .lock()
            .expect("Cannot write to file")
            .write_all(buf)?;
        io::stderr().write_all(buf)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.lock().expect("Cannot flush file").flush()?;
        io::stderr().flush()
    }
}

fn set_logger() {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("server.log")
        .expect("Failed to open log file");

    let writer = MultiWriter {
        file: Mutex::new(file),
    };

    Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format(|buf, record| {
            let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S,%3f");

            writeln!(
                buf,
                "{} - {} - {} - {}",
                ts,
                record.module_path().unwrap_or_else(|| record.target()),
                record.level(),
                record.args()
            )
        })
        .target(Target::Pipe(Box::new(writer)))
        .init();
}

fn get_remote_addr(request: &Request<grpc_pb::Numbers>) -> String {
    request
        .remote_addr()
        .unwrap_or_else(|| "unknown".parse().expect("Failed to parse address"))
        .to_string()
}

#[derive(Default)]
struct MyCalculator;

#[tonic::async_trait]
impl Calculator for MyCalculator {
    async fn add(
        &self,
        request: Request<grpc_pb::Numbers>,
    ) -> Result<Response<grpc_pb::Result>, Status> {
        let remote_addr = get_remote_addr(&request);
        let r = request.into_inner();
        info!(
            "Received adding [{}] {} and {}",
            remote_addr, r.num1, r.num2
        );
        let result = i64::from(r.num1) + i64::from(r.num2);
        info!("Send result of addition [{remote_addr}]: {result}");
        Ok(Response::new(grpc_pb::Result { result }))
    }

    async fn sub(
        &self,
        request: Request<grpc_pb::Numbers>,
    ) -> Result<Response<grpc_pb::Result>, Status> {
        let remote_addr = get_remote_addr(&request);
        let r = request.into_inner();
        info!(
            "Received subtracting [{}] {} and {}",
            remote_addr, r.num1, r.num2
        );
        let result = i64::from(r.num1) - i64::from(r.num2);
        info!("Send result of subtraction [{remote_addr}]: {result}");
        Ok(Response::new(grpc_pb::Result { result }))
    }

    async fn mul(
        &self,
        request: Request<grpc_pb::Numbers>,
    ) -> Result<Response<grpc_pb::Result>, Status> {
        let remote_addr = get_remote_addr(&request);
        let r = request.into_inner();
        info!(
            "Received multiplying [{}] {} and {}",
            remote_addr, r.num1, r.num2
        );
        let result = i64::from(r.num1) * i64::from(r.num2);
        info!("Send result of multiplication [{remote_addr}]: {result}");
        Ok(Response::new(grpc_pb::Result { result }))
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
    let args = Args::parse();
    set_logger();

    info!("Starting server on {}:{}", args.host, args.port);
    let addr = format!("{}:{}", args.host, args.port).parse()?;
    let svc = CalculatorServer::new(MyCalculator);
    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
