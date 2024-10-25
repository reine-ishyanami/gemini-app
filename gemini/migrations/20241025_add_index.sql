PRAGMA foreign_keys = ON;
-- ----------------------------
-- add index of conversation_modify_time for gemini_conversation
-- ----------------------------
CREATE INDEX IF NOT EXISTS "idx_gemini_conversation_modify_time" ON "gemini_conversation" ("conversation_modify_time");

PRAGMA foreign_keys = OFF;
