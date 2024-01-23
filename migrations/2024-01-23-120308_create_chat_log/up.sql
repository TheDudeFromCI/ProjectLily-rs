-- Your SQL goes here
CREATE TABLE `chat_log`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	`role` VARCHAR NOT NULL,
	`user` VARCHAR,
	`content` TEXT NOT NULL,
	`action` VARCHAR,
	`severity` VARCHAR,
	`time` TIMESTAMP NOT NULL,
	`token_count` INT2 NOT NULL
);
