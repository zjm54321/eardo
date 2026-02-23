-- 用户角色枚举
CREATE TYPE user_role AS ENUM ('Admin', 'User', 'FakeUser', 'BannedUser', 'DeletedUser', 'Bot');

-- 认证类型枚举
CREATE TYPE auth_type_enum AS ENUM (
    'password_email', 
    'password_phone', 
    'oauth_github', 
    'oauth_wechat', 
    'passkey'
);

-- 用户基本信息表 (对应 User 和 UserMeta 结构体)
CREATE TABLE "user" (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nick_name VARCHAR(255) NOT NULL,
    avatar BYTEA, -- 存储头像的二进制数据
    bio TEXT NOT NULL DEFAULT '',
    role user_role NOT NULL DEFAULT 'User',
    level INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 用户认证信息表 (对应 UserAuth 枚举及其内部结构)
-- 支持一个用户绑定多种登录方式（如邮箱密码、手机号密码、GitHub、微信、Passkey等）
CREATE TABLE user_auth (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    
    -- 认证类型: 'password_email', 'password_phone', 'oauth_github', 'oauth_wechat', 'passkey'
    auth_type auth_type_enum NOT NULL, 
    
    -- 认证标识: 邮箱地址、手机号、第三方平台的用户ID、Passkey的ID等
    provider_id VARCHAR(255) NOT NULL, 
    
    -- 凭证数据: 密码哈希值 (password_hash) 或 Passkey 的公钥等数据
    credential TEXT, 
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 确保同一种登录方式下的标识是唯一的（例如同一个邮箱只能注册一次）
    UNIQUE(auth_type, provider_id),
    -- 确保一个用户不能绑定两个相同的认证类型（例如一个用户不能绑定两个邮箱）
    UNIQUE(user_id, auth_type)
);

-- 创建索引以加速查询
CREATE INDEX idx_user_auth_user_id ON user_auth(user_id);
CREATE INDEX idx_user_auth_provider_id ON user_auth(provider_id);

-- 用户会话表 (用于管理短期登录凭证/Session)
CREATE TABLE user_session (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- Session ID，将作为 Cookie 发送给浏览器
    user_id UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    
    -- 记录用户是通过哪种方式登录的
    auth_type auth_type_enum NOT NULL,
    
    -- 客户端信息 
    user_agent VARCHAR(512),
    ip_address INET,
    
    -- 会话过期时间
    expires_at TIMESTAMPTZ NOT NULL,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引以加速会话查询和清理过期会话
CREATE INDEX idx_user_session_user_id ON user_session(user_id);
CREATE INDEX idx_user_session_expires_at ON user_session(expires_at);

-- 自动更新 updated_at 字段的触发器函数
CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 为 user 表添加触发器
CREATE TRIGGER update_user_modtime
    BEFORE UPDATE ON "user"
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- 为 user_auth 表添加触发器
CREATE TRIGGER update_user_auth_modtime
    BEFORE UPDATE ON user_auth
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- 为 user_session 表添加触发器
CREATE TRIGGER update_user_session_modtime
    BEFORE UPDATE ON user_session
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- 自动清理过期会话的函数，每天午夜执行一次
-- 注意：这需要数据库管理员预先安装并启用 pg_cron 扩展
CREATE OR REPLACE FUNCTION cleanup_expired_sessions()
RETURNS void AS $$
BEGIN
    DELETE FROM user_session WHERE expires_at < NOW();
END;
$$ LANGUAGE plpgsql;

CREATE EXTENSION IF NOT EXISTS pg_cron;
SELECT cron.schedule('cleanup_sessions', '0 0 * * *', 'SELECT cleanup_expired_sessions()'); 
