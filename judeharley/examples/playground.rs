use judeharley::Decimal;

#[tokio::main]
async fn main() {
    let db = judeharley::connect_database("postgres://byers:byers@localhost/byers").await.unwrap();

    let songs = judeharley::Songs::search("zehanpuryu", &db).await.unwrap();

    println!("{:#?}", songs);
}
