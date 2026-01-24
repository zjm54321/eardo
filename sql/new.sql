-- 开启外键支持
PRAGMA foreign_keys = ON;

-- ==========================================
-- 1. 用户核心资料表 (Users - Public Profile)
-- ==========================================
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,        -- UUID
    
    -- 公开资料
    nickname TEXT,                       -- 昵称
    bio TEXT,                            -- 简介
    level INTEGER DEFAULT 1,             -- 等级
    
    -- 头像资源
    avatar_data BLOB,
    avatar_mime TEXT,
    
    -- normal: 正常, deleted: 注销, banned: 封禁
    status TEXT DEFAULT 'normal' NOT NULL,
    
    role TEXT DEFAULT 'user',            -- user, admin
    
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);


-- ==========================================
-- 2. 用户鉴权表 (User Auth - Private)
-- ==========================================
CREATE TABLE IF NOT EXISTS user_auth (
    user_id TEXT PRIMARY KEY NOT NULL,   -- 与 users.id 一致
    
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    email TEXT UNIQUE,
    phone TEXT UNIQUE,
    
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_auth_username ON user_auth(username);
CREATE INDEX IF NOT EXISTS idx_auth_email ON user_auth(email);
CREATE INDEX IF NOT EXISTS idx_auth_phone ON user_auth(phone);


-- ==========================================
-- 3. 会话管理表 (Sessions)
-- ==========================================
CREATE TABLE IF NOT EXISTS user_sessions (
    token TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    
    ip_address TEXT,
    user_agent TEXT,
    device_name TEXT,
    
    expires_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_accessed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);


-- ==========================================
-- 4. 用户上传源声音 (User Source Voices)
-- (存储用户上传的原始干音/数据集，保留作为资产)
-- ==========================================
CREATE TABLE IF NOT EXISTS user_voices (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    
    audio_data BLOB NOT NULL,
    visualization_data BLOB,
    
    status TEXT DEFAULT 'normal' NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);


-- ==========================================
-- 5. 声音模型表 (Voice Models)
-- (系统底模库，现在的元信息必须基于此表)
-- ==========================================
CREATE TABLE IF NOT EXISTS voice_models (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    description TEXT,
    
    icon_data BLOB,
    
    status TEXT DEFAULT 'normal' NOT NULL, -- normal, hidden
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);


-- ==========================================
-- 6. 元信息表 (Voice Meta Infos)
-- 【核心变更】：移除了 type，强制关联 voice_models
-- ==========================================
CREATE TABLE IF NOT EXISTS voice_meta_infos (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    
    name TEXT NOT NULL,
    description TEXT,
    
    -- 【修改】直接关联声音模型表
    base_model_id TEXT NOT NULL,
    
    -- 参数
    pitch REAL DEFAULT 0.0,
    speed REAL DEFAULT 1.0,
    volume REAL DEFAULT 1.0,
    emotion TEXT DEFAULT 'normal',
    
    usage_count INTEGER DEFAULT 0,
    is_public BOOLEAN DEFAULT 1,
    
    status TEXT DEFAULT 'normal' NOT NULL,
    
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    -- 【新增约束】确保使用的模型 ID 真实存在于 voice_models 表中
    FOREIGN KEY(base_model_id) REFERENCES voice_models(id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_metas_usage ON voice_meta_infos(usage_count DESC);
CREATE INDEX IF NOT EXISTS idx_metas_plaza ON voice_meta_infos(is_public, status);
CREATE INDEX IF NOT EXISTS idx_metas_model ON voice_meta_infos(base_model_id);


-- ==========================================
-- 7. 帖子表 (Posts)
-- ==========================================
CREATE TABLE IF NOT EXISTS posts (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    
    title TEXT NOT NULL,
    content TEXT,
    
    generated_audio_data BLOB NOT NULL,
    
    voice_meta_id TEXT NOT NULL,
    
    likes_count INTEGER DEFAULT 0,
    comments_count INTEGER DEFAULT 0,
    
    status TEXT DEFAULT 'normal' NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY(voice_meta_id) REFERENCES voice_meta_infos(id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_posts_created ON posts(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_posts_status ON posts(status);


-- ==========================================
-- 8. 评论表 (Post Comments)
-- ==========================================
CREATE TABLE IF NOT EXISTS post_comments (
    id TEXT PRIMARY KEY NOT NULL,
    post_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    
    content TEXT NOT NULL,
    
    status TEXT DEFAULT 'normal' NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY(post_id) REFERENCES posts(id) ON DELETE CASCADE,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_comments_post ON post_comments(post_id);


-- ==========================================
-- 9. 点赞表 (Post Likes)
-- ==========================================
CREATE TABLE IF NOT EXISTS post_likes (
    user_id TEXT NOT NULL,
    post_id TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (user_id, post_id),
    
    FOREIGN KEY(post_id) REFERENCES posts(id) ON DELETE CASCADE,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);