-- Applied by deploy/bin/eardo-rollout inside one transaction: do not add BEGIN/COMMIT.
-- Beijing live probes: the old/datestamped instruct IDs return Model not exist.
-- Preserve voice_model primary keys, voice IDs, voice_meta, and generated audio.
-- Qwen3 Flash supports these system voices but does not support instructions.

UPDATE voice_model
SET supported_model = 'qwen3-tts-flash',
    instruction_control = false
WHERE supported_model IN (
    'qwen3-tts-instruct-flash',
    'qwen3-tts-instruct-flash-2026-01-26'
);

UPDATE voice_model
SET instruction_control = false
WHERE supported_model = 'qwen3-tts-flash'
  AND instruction_control = true;

-- VC/VD rows are intentionally not reassigned to a different speaker: they
-- require provider enrollment. The backend returns an explicit enrollment error.
