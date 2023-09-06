use testcontainers::{clients, core::WaitFor, images::postgres::Postgres};
use tokio::test;
use tokio_postgres::Row;

#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub email: String,
}

impl From<Row> for User {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            username: row.get("username"),
            password: row.get("password"),
            email: row.get("email"),
        }
    }
}

#[test]
async fn it_works() {
    let docker = clients::Cli::default();

    // Define a PostgreSQL container image
    let postgres_image = Postgres::default();

    let pg_container = docker.run(postgres_image);

    pg_container.start();

    WaitFor::seconds(60);

    // Get the PostgreSQL port
    let pg_port = pg_container.get_host_port_ipv4(5432);

    // Define the connection to the Postgress client
    let (client, connection) = tokio_postgres::Config::new()
        .user("postgres")
        .password("postgres")
        .host("localhost")
        .port(pg_port)
        .dbname("postgres")
        .connect(tokio_postgres::NoTls)
        .await
        .unwrap();

    // Spawn connection
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("Connection error: {}", error);
        }
    });

    let _ = client
        .batch_execute(
            "
        CREATE TABLE IF NOT EXISTS app_user (
            id              SERIAL PRIMARY KEY,
            username        VARCHAR UNIQUE NOT NULL,
            password        VARCHAR NOT NULL,
            email           VARCHAR UNIQUE NOT NULL
            )
    ",
        )
        .await;

    let _ = client
        .execute(
            "INSERT INTO app_user (username, password, email) VALUES ($1, $2, $3)",
            &[&"user1", &"mypass", &"user@test.com"],
        )
        .await;

    let result = client
        .query("SELECT id, username, password, email FROM app_user", &[])
        .await
        .unwrap();

    let users: Vec<User> = result.into_iter().map(|row| User::from(row)).collect();

    let user = users.first().unwrap();

    assert_eq!(1, user.id);
    assert_eq!("user1", user.username);
    assert_eq!("mypass", user.password);
    assert_eq!("user@test.com", user.email);
}
