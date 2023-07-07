use actix_web::web::Buf;
use actix_web::{get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::StreamExt;
use lazy_static::lazy_static;
use rand::Rng;
use std::io::Write;
use std::process::Command;
use std::{env, iter};

fn random_string(len: usize) -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_";
    let mut rng = rand::thread_rng();
    let one_char = || CHARSET[rng.gen_range(0..CHARSET.len())] as char;
    iter::repeat_with(one_char).take(len).collect()
}

fn temp_file(ext: &str) -> String {
    std::format!(
        "{}{}.{}",
        env::temp_dir().to_str().unwrap(),
        random_string(32),
        ext
    )
}

lazy_static! {
    static ref ARGS: Vec<String> = env::var("ARGS").unwrap().split('$').map(|s| s.to_string()).collect();
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/sign")]
async fn sign(req: HttpRequest, mut payload: web::Payload) -> Result<HttpResponse, Error> {
    let inpath = temp_file("dll");
    let outpath = temp_file("dll");

    {
        let mut infile = std::fs::File::create(&inpath)?;
        while let Some(chunk) = payload.next().await {
            let chunk = chunk?;
            infile.write_all(chunk.chunk())?;
        }
    }

    let out = Command::new("osslsigncode")
        .arg("sign")
        .args(ARGS.iter())
        .arg("-in")
        .arg(inpath.as_str())
        .arg("-out")
        .arg(outpath.as_str())
        .output();

    std::fs::remove_file(inpath)?;

    let out = out?;

    Ok(if out.status.success() {
        let file = actix_files::NamedFile::open_async(&outpath)
            .await?
            .into_response(&req);
        let _ = std::fs::remove_file(outpath);
        file
    } else {
        let mut body = format!(
            "osslsigncode exited with error {}:\n",
            out.status.code().unwrap()
        )
        .into_bytes();
        body.extend(out.stdout);
        body.extend(out.stderr);
        HttpResponse::InternalServerError().body(body)
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host = env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port = str::parse::<u16>(env::var("PORT").unwrap_or("8080".to_string()).as_str()).unwrap();

    HttpServer::new(|| App::new().service(hello).service(sign))
        .bind((host, port))?
        .run()
        .await
}
