
CREATE TABLE IF NOT EXISTS "cinema" (
	"id" text CHECK (length(id)=14) not null,
   	"name" text not null,
   	"updated_on" timestamp not null,
    "data" json not null,
   	PRIMARY KEY ("id")
);

CREATE TABLE IF NOT EXISTS "cinema_progress" (
	"entry_id" text CHECK (length(entry_id)=14) not null,
	"user_id" text CHECK (length(user_id)=14) not null,
	"updated_on" timestamp not null,
	"data" json not null,
	PRIMARY KEY ("entry_id", "user_id")
);
