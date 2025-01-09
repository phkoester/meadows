// str.rs

//! String-related utilities.

// `StrExt` -------------------------------------------------------------------------------------------------

/// An extension trait for strings.
///
/// This is included in the crate's [prelude](crate::prelude).
pub trait StrExt {
  /// Creates a new [`String`] by enclosing this string in back ticks.
  ///
  /// # Examples
  ///
  /// ```
  /// use meadows::prelude::*;
  ///
  /// assert_eq!("name".bt(), "`name`");
  /// ```
  #[must_use]
  fn bt(&self) -> String;

  /// Creates a new [`String`] by converting the first [`char`] of this string to uppercase.
  ///
  /// # Examples
  ///
  /// ```
  /// use meadows::prelude::*;
  ///
  /// assert_eq!("übermut".capitalize(), "Übermut");
  /// ```
  #[must_use]
  fn capitalize(&self) -> String;

  /// Creates a new [`String`] by putting this string, which may be a multi-line string, into a fence that is
  /// made up of `c` and `text_width` - 1 characters wide.
  ///
  /// # Examples
  ///
  /// ```
  /// use meadows::prelude::*;
  ///
  /// assert_eq!("1st line\n2nd line".fence('*', 8), "*******\n*\n* 1st line\n* 2nd line\n*\n*******");
  /// ```
  #[must_use]
  fn fence(&self, c: char, text_width: usize) -> String;

  /// Creates a new [`String`] by converting the first [`char`] of this string to lowercase.
  ///
  /// # Examples
  ///
  /// ```
  /// use meadows::prelude::*;
  ///
  /// assert_eq!("Übermut".uncapitalize(), "übermut");
  /// ```
  #[must_use]
  fn uncapitalize(&self) -> String;
}

impl StrExt for str {
  #[inline]
  fn bt(&self) -> String { format!("`{self}`") }

  fn capitalize(&self) -> String {
    let mut it = self.chars();

    match it.next() {
      None => String::new(),
      Some(c) => c.to_uppercase().collect::<String>() + it.as_str(),
    }
  }

  fn fence(&self, c: char, text_width: usize) -> String {
    let mut ret = String::new();

    let row = c.to_string().repeat(text_width - 1);

    ret.push_str(&row);
    ret.push('\n');
    ret.push(c);
    ret.push('\n');

    for line in self.lines() {
      ret.push(c);
      ret.push(' ');
      ret.push_str(line);
      ret.push('\n');
    }

    ret.push(c);
    ret.push('\n');
    ret.push_str(&row);

    ret
  }

  fn uncapitalize(&self) -> String {
    let mut it = self.chars();

    match it.next() {
      None => String::new(),
      Some(c) => c.to_lowercase().collect::<String>() + it.as_str(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // `StrExt` -----------------------------------------------------------------------------------------------

  #[test]
  fn test_str_ext_bt() {
    assert_eq!("a".bt(), "`a`");
    assert_eq!("äöü".to_owned().bt(), "`äöü`");
  }

  #[test]
  fn test_str_ext_capitalize() {
    assert_eq!("".capitalize(), "");
    assert_eq!("abc".capitalize(), "Abc");
    assert_eq!("äöü".capitalize(), "Äöü");
    assert_eq!("€".capitalize(), "€");
  }

  #[test]
  fn test_str_ext_uncapitalize() {
    assert_eq!("".uncapitalize(), "");
    assert_eq!("Abc".uncapitalize(), "abc");
    assert_eq!("Äöü".uncapitalize(), "äöü");
    assert_eq!("€".uncapitalize(), "€");
  }
}

// EOF
