entity:
# sea generate entity -o judeharley/src/entities -u postgres://postgres:abcd1234@localhost/postgres
	sea generate entity -o caliborn/src/entities -u postgres://postgres:abcd1234@localhost/postgres
migrate:
	sea migrate up -u postgres://postgres:abcd1234@localhost/postgres

undo:
	sea migrate down -u postgres://postgres:abcd1234@localhost/postgres

.PHONY: entity