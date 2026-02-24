-- 语音模型分类枚举
CREATE TYPE voice_model_category_enum AS ENUM ('Official', 'User');

-- 语音模型表 (对应 VoiceModel 结构体)
CREATE TABLE voice_model (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    voice_id VARCHAR(255) NOT NULL, -- 外部系统的语音ID，可能重复
    
    -- VoiceModelInfo (使用 sqlx flatten 映射)
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    
    avatar BYTEA, -- 存储头像的二进制数据
    
    -- VoiceModelCategory (使用 sqlx flatten 映射)
    category voice_model_category_enum NOT NULL,
    supported_model TEXT NOT NULL DEFAULT '', -- 当 category 为 Official 时使用的标签，或当 category 为 User 时支持的模型
    user_id UUID REFERENCES "user"(id) ON DELETE CASCADE, -- 当 category 为 User 时的用户ID
    voice_generate_data_id UUID, -- 当 category 为 User 时的生成数据ID
    
    -- VoiceModelAbility (使用 sqlx flatten 映射)
    voice_clone BOOLEAN NOT NULL DEFAULT false,
    voice_design BOOLEAN NOT NULL DEFAULT false,
    ssml BOOLEAN NOT NULL DEFAULT false,
    latex BOOLEAN NOT NULL DEFAULT false,
    parametric_control BOOLEAN NOT NULL DEFAULT false,
    instruction_control BOOLEAN NOT NULL DEFAULT false,
    
    score INTEGER NOT NULL DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引以加速查询
CREATE INDEX idx_voice_model_voice_id ON voice_model(voice_id);
CREATE INDEX idx_voice_model_category ON voice_model(category);
CREATE INDEX idx_voice_model_user_id ON voice_model(user_id);
CREATE INDEX idx_voice_model_name ON voice_model(name);
CREATE INDEX idx_voice_model_score ON voice_model(score DESC);

-- 语音元数据表 (对应 VoiceMeta 结构体)
-- 用于存储语音模型的自定义参数配置
CREATE TABLE voice_meta (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    voice_model_id UUID NOT NULL REFERENCES voice_model(id) ON DELETE CASCADE,
    
    -- Parametic (使用 sqlx flatten 映射)
    pitch REAL,
    speed REAL,
    volume REAL,
    
    -- Instruction
    instruction TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 声音库表 (存储生成的语音记录)
CREATE TABLE voice_library (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    voice_meta_id UUID NOT NULL REFERENCES voice_meta(id) ON DELETE CASCADE,
    
    -- 文本内容
    text_content TEXT NOT NULL,
    
    -- 声音文件 (存储二进制数据)
    audio_data BYTEA NOT NULL,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引以加速声音库查询
CREATE INDEX idx_voice_library_voice_meta_id ON voice_library(voice_meta_id);

-- 为 voice_model 表添加触发器
CREATE TRIGGER update_voice_model_modtime
    BEFORE UPDATE ON voice_model
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- 为 voice_meta 表添加触发器
CREATE TRIGGER update_voice_meta_modtime
    BEFORE UPDATE ON voice_meta
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();