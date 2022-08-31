use payment_engine::cmd;

#[rocket::main]
async fn main() {
    std::process::exit(cmd::set_up_logger_and_exec().await)
}
