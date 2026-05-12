UPDATE tournament SET kind = 'bot' WHERE kind = NONE;
UPDATE match SET faulted_user_ids = [] WHERE faulted_user_ids = NONE;
UPDATE match SET participants = array::map(
    participants,
    |$p| {
        user_id: $p.user_id,
        submission_id: $p.submission_id,
        score: IF $p.score = NONE THEN NONE ELSE <float> $p.score END
    }
) WHERE participants[*].score IS NOT NONE;
