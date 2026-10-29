pub fn fuzzy_score(query: &str, candidate: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }

    let query_lower: Vec<char> = query.to_lowercase().chars().collect();
    let candidate_lower: Vec<char> = candidate.to_lowercase().chars().collect();

    let mut q_idx = 0;
    let mut score = 0;
    let mut consecutive = 0;
    let mut last_match_idx = None;

    for (c_idx, &c_char) in candidate_lower.iter().enumerate() {
        if q_idx < query_lower.len() && c_char == query_lower[q_idx] {
            score += 10;
            if let Some(last) = last_match_idx {
                if c_idx == last + 1 {
                    consecutive += 5;
                    score += consecutive;
                } else {
                    consecutive = 0;
                }
            } else {
                score += 20;
            }

            if c_idx == 0 || matches!(candidate_lower[c_idx - 1], ':' | '_' | '/' | '-') {
                score += 15;
            }

            last_match_idx = Some(c_idx);
            q_idx += 1;
        }
    }

    if q_idx == query_lower.len() {
        let len_penalty = (candidate.len() as i32 - query.len() as i32).max(0);
        Some(score - len_penalty / 2)
    } else {
        None
    }
}
