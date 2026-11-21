# Size & Date Parsers Reference

Details on converting user-facing size strings and date keywords into exact byte integers and UNIX epoch seconds.

---

## 1. File Size Unit Conversion

Unit | Multiplier (Binary Standard: 1024) | Example
:--- | :--- | :---
`b` / `bytes` | $1$ byte | `size:>500b`
`k` / `kb` | $1024$ bytes ($2^{10}$) | `size:<64kb`
`m` / `mb` | $1,048,576$ bytes ($2^{20}$) | `size:>15mb`
`g` / `gb` | $1,073,741,824$ bytes ($2^{30}$) | `size:>2gb`
`empty` | $0$ bytes | `size:empty`

### Parsing Algorithm:
```rust
pub fn parse_size_bytes(s: &str) -> Option<u64> {
    let s = s.trim().to_lowercase();
    if s == "empty" {
        return Some(0);
    }
    let (num_part, unit_part) = split_num_and_unit(&s);
    let num: f64 = num_part.parse().ok()?;
    let multiplier = match unit_part {
        "gb" | "g" => 1024.0 * 1024.0 * 1024.0,
        "mb" | "m" => 1024.0 * 1024.0,
        "kb" | "k" => 1024.0,
        "b" | "" => 1.0,
        _ => return None,
    };
    Some((num * multiplier) as u64)
}
```

---

## 2. Date Math & Relative Epoch Modifiers

Keyword | Time Range Defined
:--- | :---
`dm:today` | From midnight today ($00:00:00$ local/UTC) to current time.
`dm:yesterday` | From midnight yesterday to $23:59:59$ yesterday.
`dm:lastweek` | Past 7 full days from current epoch seconds: $[T - 7 \times 86400, T]$.
`dm:thismonth` | First day of current calendar month at $00:00:00$ to now.
`dm:YYYY-MM-DD` | Exact 24-hour window for the specified calendar date.
