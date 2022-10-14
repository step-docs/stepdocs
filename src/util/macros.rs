#[macro_export]
macro_rules! read_or_none {
    ($self:ident, $line:ident) => {
		if 0 == $self.inner.read_line(&mut $line).await? {
			tracing::trace!("EOF");
			return Ok(None);
		}
		if $line.ends_with(|it:char| it.is_whitespace()) {
	        let _ = $line.pop();
		}
    };
}

#[macro_export]
macro_rules! read_async {
    ($self:ident, $line:ident) => {
	    {
		    $self.inner.read_line(&mut $line).await?
	    }
    };
}