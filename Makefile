entity:
	sea generate entity -o judeharley/src/entities -u postgres://byers:byers@localhost/byers
migrate:
	sea migrate up -u postgres://byers:byers@localhost/byers

undo:
	sea migrate down -u postgres://byers:byers@localhost/byers

.PHONY: entity