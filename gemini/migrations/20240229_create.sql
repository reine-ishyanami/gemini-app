PRAGMA foreign_keys = ON;
-- ----------------------------
-- Table structure for gemini_conversation
-- ----------------------------
CREATE TABLE IF NOT EXISTS "gemini_conversation" (
  "conversation_id" TEXT NOT NULL,
  "conversation_title" TEXT,
  "conversation_start_time" DATE,
  "conversation_modify_time" DATE,
  PRIMARY KEY ("conversation_id")
);
-- ----------------------------
-- Table structure for gemini_message_record
-- ----------------------------
CREATE TABLE IF NOT EXISTS "gemini_message_record" (
  "record_id" TEXT NOT NULL,
  "conversation_id" TEXT,
  "record_content" TEXT,
  "record_time" DATE,
  "record_sender" TEXT,
  "sort_index" INTEGER DEFAULT 0,
  PRIMARY KEY ("record_id"),
  FOREIGN KEY ("conversation_id") REFERENCES "gemini_conversation" ("conversation_id") ON DELETE CASCADE ON UPDATE CASCADE
);
-- ----------------------------
-- Table structure for gemini_image_record
-- ----------------------------
CREATE TABLE IF NOT EXISTS "gemini_image_record" (
  "image_record_id" TEXT NOT NULL,
  "record_id" TEXT,
  "image_path" TEXT,
  "image_type" TEXT,
  "image_base64" TEXT,
  PRIMARY KEY ("image_record_id"),
  FOREIGN KEY ("record_id") REFERENCES "gemini_message_record" ("record_id") ON DELETE CASCADE ON UPDATE CASCADE
);
PRAGMA foreign_keys = OFF;
