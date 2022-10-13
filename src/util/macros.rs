#[macro_export]
macro_rules! read_or_none {
    ($self:ident, $line:ident) => {
		if 0 == tokio::io::AsyncBufReadExt::read_line(&mut $self.inner, &mut $line).await? {
			tracing::trace!("EOF");
			return Ok(None);
		}
		if $line.ends_with(|it:char| it.is_whitespace()) {
	        let _ = $line.pop();
		}
    };
}