// Phase 0: Removes `#` and `//` comments while retaining evry newline and width of
// each line so line numbers dont get messed up in visualizer. Our language dosent have
// string literals so comment markers are never ambigous with quoted text.
pub fn strip_comments(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    for segment in source.split_inclusive('\n') {
        let (line, newline) = match segment.strip_suffix('\n') {
            Some(line) => (line, "\n"),
            None => (segment, ""),
        };
        let mut cut = line.len();
        for (index, ch) in line.char_indices() {
            if ch == '#' || (ch == '/' && line[index..].starts_with("//")) {
                cut = index;
                break;
            }
        }
        output.push_str(&line[..cut]);
        let comment_len = line.len().saturating_sub(cut);
        output.push_str(&" ".repeat(comment_len));
        output.push_str(newline);
    }
    output
}
