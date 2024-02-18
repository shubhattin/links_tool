CREATE TABLE IF NOT EXISTS "links" (
	"id" varchar(20) PRIMARY KEY NOT NULL,
	"enabled" boolean DEFAULT true NOT NULL,
	"link" text NOT NULL,
	"prefix_zeros" integer DEFAULT 0 NOT NULL,
	"name" varchar(30)
);
--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "others" (
	"key" varchar(20) PRIMARY KEY NOT NULL,
	"value" text NOT NULL
);
