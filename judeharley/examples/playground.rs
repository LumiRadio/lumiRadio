use num_traits::FromPrimitive;

#[tokio::main]
async fn main() {
    let db = judeharley::connect_database("postgres://byers:byers@localhost/byers").await.unwrap();
}
