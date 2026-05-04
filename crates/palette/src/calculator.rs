/// Évalue une expression mathématique ou une conversion d'unités.
/// Retourne None si le texte n'est pas reconnu.
pub fn evaluate(input: &str) -> Option<String> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    // Essaie d'abord les conversions : "X unit in unit2"
    if let Some(result) = try_conversion(input) {
        return Some(result);
    }

    // Ensuite l'arithmétique
    if let Some(result) = try_math(input) {
        return Some(format_number(result));
    }

    None
}

// ── Conversions ───────────────────────────────────────────────────────────────

fn try_conversion(input: &str) -> Option<String> {
    // Patterns : "X unit1 in unit2", "X unit1 to unit2", "X unit1 en unit2"
    let lower = input.to_lowercase();
    let sep = if lower.contains(" in ") {
        " in "
    } else if lower.contains(" to ") {
        " to "
    } else if lower.contains(" en ") {
        " en "
    } else {
        return None;
    };

    let parts: Vec<&str> = lower.splitn(2, sep).collect();
    if parts.len() != 2 {
        return None;
    }

    let from_str = parts[0].trim();
    let to_unit = parts[1].trim();

    // Sépare la valeur numérique de l'unité source
    let (value, from_unit) = split_value_unit(from_str)?;

    convert(value, from_unit, to_unit)
}

fn split_value_unit(s: &str) -> Option<(f64, &str)> {
    let s = s.trim();
    // Trouve la limite entre chiffres et lettres
    let split_at = s
        .char_indices()
        .find(|(_, c)| c.is_alphabetic() || *c == '°')
        .map(|(i, _)| i)?;
    let num_str = s[..split_at].trim();
    let unit = s[split_at..].trim();
    let value: f64 = num_str.parse().ok()?;
    Some((value, unit))
}

fn convert(value: f64, from: &str, to: &str) -> Option<String> {
    // Normalise en SI puis convertit
    let (si_value, base_unit) = to_si(value, from)?;
    let result = from_si(si_value, to, base_unit)?;
    Some(format!("{} {} = {} {}", format_number(value), from, format_number(result), to))
}

/// Convertit vers l'unité SI de base, retourne (valeur_si, famille)
fn to_si(v: f64, unit: &str) -> Option<(f64, &'static str)> {
    match unit {
        // Longueur → mètres
        "km" => Some((v * 1000.0, "length")),
        "m" => Some((v, "length")),
        "cm" => Some((v / 100.0, "length")),
        "mm" => Some((v / 1000.0, "length")),
        "mi" | "miles" | "mile" => Some((v * 1609.344, "length")),
        "ft" | "feet" | "foot" => Some((v * 0.3048, "length")),
        "in" | "inch" | "inches" => Some((v * 0.0254, "length")),
        "yd" | "yards" | "yard" => Some((v * 0.9144, "length")),
        // Masse → kilogrammes
        "kg" => Some((v, "mass")),
        "g" => Some((v / 1000.0, "mass")),
        "lb" | "lbs" | "pound" | "pounds" => Some((v * 0.453592, "mass")),
        "oz" | "ounce" | "ounces" => Some((v * 0.0283495, "mass")),
        "t" | "tonne" | "tonnes" => Some((v * 1000.0, "mass")),
        // Volume → litres
        "l" | "L" | "litre" | "litres" | "liter" | "liters" => Some((v, "volume")),
        "ml" | "mL" => Some((v / 1000.0, "volume")),
        "cl" | "cL" => Some((v / 100.0, "volume")),
        "gal" | "gallon" | "gallons" => Some((v * 3.78541, "volume")),
        "fl oz" | "floz" => Some((v * 0.0295735, "volume")),
        // Vitesse → m/s
        "km/h" | "kmh" | "kph" => Some((v / 3.6, "speed")),
        "m/s" | "ms" => Some((v, "speed")),
        "mph" => Some((v * 0.44704, "speed")),
        "knot" | "knots" | "kt" => Some((v * 0.514444, "speed")),
        // Température — cas spécial, pas de SI
        "°c" | "c" | "celsius" => Some((v, "temp_c")),
        "°f" | "f" | "fahrenheit" => Some(((v - 32.0) * 5.0 / 9.0, "temp_c")),
        "°k" | "k" | "kelvin" => Some((v - 273.15, "temp_c")),
        _ => None,
    }
}

fn from_si(si: f64, unit: &str, family: &str) -> Option<f64> {
    match (family, unit) {
        // Longueur
        ("length", "km") => Some(si / 1000.0),
        ("length", "m") => Some(si),
        ("length", "cm") => Some(si * 100.0),
        ("length", "mm") => Some(si * 1000.0),
        ("length", "mi") | ("length", "miles") | ("length", "mile") => Some(si / 1609.344),
        ("length", "ft") | ("length", "feet") | ("length", "foot") => Some(si / 0.3048),
        ("length", "in") | ("length", "inch") | ("length", "inches") => Some(si / 0.0254),
        ("length", "yd") | ("length", "yards") | ("length", "yard") => Some(si / 0.9144),
        // Masse
        ("mass", "kg") => Some(si),
        ("mass", "g") => Some(si * 1000.0),
        ("mass", "lb") | ("mass", "lbs") | ("mass", "pound") | ("mass", "pounds") => {
            Some(si / 0.453592)
        }
        ("mass", "oz") | ("mass", "ounce") | ("mass", "ounces") => Some(si / 0.0283495),
        ("mass", "t") | ("mass", "tonne") | ("mass", "tonnes") => Some(si / 1000.0),
        // Volume
        ("volume", "l") | ("volume", "L") | ("volume", "litre") | ("volume", "litres") => Some(si),
        ("volume", "ml") | ("volume", "mL") => Some(si * 1000.0),
        ("volume", "cl") | ("volume", "cL") => Some(si * 100.0),
        ("volume", "gal") | ("volume", "gallon") | ("volume", "gallons") => Some(si / 3.78541),
        // Vitesse
        ("speed", "km/h") | ("speed", "kmh") | ("speed", "kph") => Some(si * 3.6),
        ("speed", "m/s") | ("speed", "ms") => Some(si),
        ("speed", "mph") => Some(si / 0.44704),
        ("speed", "knot") | ("speed", "knots") | ("speed", "kt") => Some(si / 0.514444),
        // Température (si = °C)
        ("temp_c", "°c") | ("temp_c", "c") | ("temp_c", "celsius") => Some(si),
        ("temp_c", "°f") | ("temp_c", "f") | ("temp_c", "fahrenheit") => {
            Some(si * 9.0 / 5.0 + 32.0)
        }
        ("temp_c", "°k") | ("temp_c", "k") | ("temp_c", "kelvin") => Some(si + 273.15),
        _ => None,
    }
}

// ── Arithmétique ──────────────────────────────────────────────────────────────

fn try_math(input: &str) -> Option<f64> {
    let mut p = Parser::new(input);
    let result = p.parse_expr().ok()?;
    // Vérifie qu'on a tout consommé
    p.skip_ws();
    if p.pos < p.input.len() {
        return None;
    }
    Some(result)
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Self { input: s, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn consume(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ') | Some('\t')) {
            self.consume();
        }
    }

    /// expr = term (('+' | '-') term)*
    fn parse_expr(&mut self) -> Result<f64, ()> {
        let mut left = self.parse_term()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('+') => {
                    self.consume();
                    left += self.parse_term()?;
                }
                Some('-') => {
                    self.consume();
                    left -= self.parse_term()?;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// term = power (('*' | '/' | '%') power)*
    fn parse_term(&mut self) -> Result<f64, ()> {
        let mut left = self.parse_power()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('*') => {
                    self.consume();
                    left *= self.parse_power()?;
                }
                Some('/') => {
                    self.consume();
                    let r = self.parse_power()?;
                    if r == 0.0 {
                        return Err(());
                    }
                    left /= r;
                }
                Some('%') => {
                    self.consume();
                    let r = self.parse_power()?;
                    left %= r;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// power = unary ('^' unary)?
    fn parse_power(&mut self) -> Result<f64, ()> {
        let base = self.parse_unary()?;
        self.skip_ws();
        if self.peek() == Some('^') {
            self.consume();
            let exp = self.parse_unary()?;
            Ok(base.powf(exp))
        } else {
            Ok(base)
        }
    }

    /// unary = '-' unary | primary
    fn parse_unary(&mut self) -> Result<f64, ()> {
        self.skip_ws();
        if self.peek() == Some('-') {
            self.consume();
            Ok(-self.parse_primary()?)
        } else {
            self.parse_primary()
        }
    }

    /// primary = '(' expr ')' | number
    fn parse_primary(&mut self) -> Result<f64, ()> {
        self.skip_ws();
        if self.peek() == Some('(') {
            self.consume();
            let val = self.parse_expr()?;
            self.skip_ws();
            if self.peek() == Some(')') {
                self.consume();
            }
            Ok(val)
        } else {
            self.parse_number()
        }
    }

    fn parse_number(&mut self) -> Result<f64, ()> {
        self.skip_ws();
        let start = self.pos;
        // Partie entière
        while matches!(self.peek(), Some('0'..='9')) {
            self.consume();
        }
        // Partie décimale
        if self.peek() == Some('.') {
            self.consume();
            while matches!(self.peek(), Some('0'..='9')) {
                self.consume();
            }
        }
        if self.pos == start {
            return Err(());
        }
        self.input[start..self.pos].parse::<f64>().map_err(|_| ())
    }
}

fn format_number(n: f64) -> String {
    if n == n.floor() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        // 6 chiffres significatifs max, sans zéros inutiles
        let s = format!("{:.6}", n);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}
