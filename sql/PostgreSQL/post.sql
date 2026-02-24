-- 帖子状态枚举
CREATE TYPE post_status AS ENUM ('Normal', 'Deleted', 'Banned', 'Recommended');

-- VoiceMetaPost 表
CREATE TABLE voice_meta_post (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    meta_id UUID NOT NULL REFERENCES voice_meta(id) ON DELETE CASCADE,
    author UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    status post_status NOT NULL DEFAULT 'Normal',
    comments_count INTEGER NOT NULL DEFAULT 0,
    likes_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- VoicePost 表
CREATE TABLE voice_post (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    library_id UUID NOT NULL REFERENCES voice_library(id) ON DELETE CASCADE,
    author UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    status post_status NOT NULL DEFAULT 'Normal',
    comments_count INTEGER NOT NULL DEFAULT 0,
    likes_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- VoiceMetaPost 评论表
CREATE TABLE voice_meta_post_comment (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES voice_meta_post(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- VoicePost 评论表
CREATE TABLE voice_post_comment (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES voice_post(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- VoiceMetaPost 点赞表
CREATE TABLE voice_meta_post_like (
    post_id UUID NOT NULL REFERENCES voice_meta_post(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (post_id, user_id)
);

-- VoicePost 点赞表
CREATE TABLE voice_post_like (
    post_id UUID NOT NULL REFERENCES voice_post(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (post_id, user_id)
);

-- 触发器
CREATE TRIGGER update_voice_meta_post_modtime
    BEFORE UPDATE ON voice_meta_post
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

CREATE TRIGGER update_voice_post_modtime
    BEFORE UPDATE ON voice_post
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- 评论和点赞数量同步触发器函数
CREATE OR REPLACE FUNCTION update_voice_meta_post_comments_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE voice_meta_post SET comments_count = comments_count + 1 WHERE id = NEW.post_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE voice_meta_post SET comments_count = comments_count - 1 WHERE id = OLD.post_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_voice_post_comments_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE voice_post SET comments_count = comments_count + 1 WHERE id = NEW.post_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE voice_post SET comments_count = comments_count - 1 WHERE id = OLD.post_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_voice_meta_post_likes_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE voice_meta_post SET likes_count = likes_count + 1 WHERE id = NEW.post_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE voice_meta_post SET likes_count = likes_count - 1 WHERE id = OLD.post_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_voice_post_likes_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE voice_post SET likes_count = likes_count + 1 WHERE id = NEW.post_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE voice_post SET likes_count = likes_count - 1 WHERE id = OLD.post_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- 绑定评论和点赞数量同步触发器
CREATE TRIGGER trigger_update_voice_meta_post_comments_count
    AFTER INSERT OR DELETE ON voice_meta_post_comment
    FOR EACH ROW
    EXECUTE FUNCTION update_voice_meta_post_comments_count();

CREATE TRIGGER trigger_update_voice_post_comments_count
    AFTER INSERT OR DELETE ON voice_post_comment
    FOR EACH ROW
    EXECUTE FUNCTION update_voice_post_comments_count();

CREATE TRIGGER trigger_update_voice_meta_post_likes_count
    AFTER INSERT OR DELETE ON voice_meta_post_like
    FOR EACH ROW
    EXECUTE FUNCTION update_voice_meta_post_likes_count();

CREATE TRIGGER trigger_update_voice_post_likes_count
    AFTER INSERT OR DELETE ON voice_post_like
    FOR EACH ROW
    EXECUTE FUNCTION update_voice_post_likes_count();
