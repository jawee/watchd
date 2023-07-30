default:
	cargo build

test:
	cargo test

resetdb:
	rm -f -- sqlite.db
	sqlx database create --database-url sqlite://sqlite.db
	sqlx migrate run --database-url sqlite://sqlite.db

