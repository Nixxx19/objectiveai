//! Shared stream-consumption helpers for snapshot tests.
//!
//! Provides [`consume_stream`] (for direct chunk streams) and
//! [`consume_stream_items`] (for `StreamItem<STATE>` streams) which use a
//! 2-behind buffer to distinguish last / second-to-last / other chunks
//! without collecting everything into a `Vec`.

use futures::StreamExt;

/// Consume a stream of chunks using a 2-behind buffer.
///
/// Instead of collecting all chunks into a `Vec`, this function holds two
/// chunks behind the current one.  When a new chunk arrives the oldest
/// buffered chunk is flushed through `on_chunk`.  After the stream ends the
/// two remaining chunks are flushed through `on_second_to_last` and
/// `on_last` respectively.
///
/// The aggregate is built incrementally: the first chunk is cloned, and
/// every subsequent chunk is merged via `push`.
///
/// # Panics
///
/// Panics if the stream yields fewer than 2 chunks (all current test
/// streams produce at least 2).
pub(crate) async fn consume_stream<C, S>(
    stream: S,
    mut push: impl FnMut(&mut C, &C),
    mut on_chunk: impl FnMut(usize, &C),
    mut on_second_to_last: impl FnMut(usize, &C),
    mut on_last: impl FnMut(usize, &C),
) -> C
where
    C: Clone,
    S: futures::Stream<Item = C> + Unpin,
{
    futures::pin_mut!(stream);

    let mut agg: Option<C> = None;
    // 2-behind buffer: (prev_prev, prev).  Indices track the chunk number.
    let mut buf: (Option<(usize, C)>, Option<(usize, C)>) = (None, None);
    let mut idx: usize = 0;

    while let Some(chunk) = stream.next().await {
        // Accumulate
        match &mut agg {
            Some(a) => push(a, &chunk),
            None => agg = Some(chunk.clone()),
        }

        // Shift buffer — flush prev_prev through on_chunk
        if let (Some((pp_idx, pp)), _) = &buf {
            // We have two buffered; flush the oldest
            on_chunk(*pp_idx, pp);
        }
        buf = (buf.1.take(), Some((idx, chunk)));
        idx += 1;
    }

    // Flush remaining buffer
    match buf {
        (Some((pp_idx, pp)), Some((p_idx, p))) => {
            on_second_to_last(pp_idx, &pp);
            on_last(p_idx, &p);
        }
        (None, Some((p_idx, p))) => {
            // Only one chunk total — treat it as both second-to-last and last.
            // (In practice our streams always have ≥2 chunks.)
            on_last(p_idx, &p);
        }
        _ => panic!("stream must produce at least one chunk"),
    }

    agg.expect("stream must produce at least one chunk")
}

/// Like [`consume_stream`], but for streams whose items are not directly
/// the chunk type (e.g. `StreamItem<STATE>` which wraps chunks alongside
/// state items).
///
/// `extract` converts each stream item into `Some(chunk)` or `None`.
/// Non-chunk items (`None`) are counted; after the stream ends, the
/// function asserts that at least one non-chunk item was seen.
pub(crate) async fn consume_stream_items<C, I, S>(
    stream: S,
    mut extract: impl FnMut(I) -> Option<C>,
    mut push: impl FnMut(&mut C, &C),
    mut on_chunk: impl FnMut(usize, &C),
    mut on_second_to_last: impl FnMut(usize, &C),
    mut on_last: impl FnMut(usize, &C),
) -> C
where
    C: Clone,
    S: futures::Stream<Item = I> + Unpin,
{
    futures::pin_mut!(stream);

    let mut agg: Option<C> = None;
    let mut buf: (Option<(usize, C)>, Option<(usize, C)>) = (None, None);
    let mut idx: usize = 0;
    let mut saw_non_chunk = false;

    while let Some(item) = stream.next().await {
        match extract(item) {
            Some(chunk) => {
                // Accumulate
                match &mut agg {
                    Some(a) => push(a, &chunk),
                    None => agg = Some(chunk.clone()),
                }

                // Shift buffer
                if let (Some((pp_idx, pp)), _) = &buf {
                    on_chunk(*pp_idx, pp);
                }
                buf = (buf.1.take(), Some((idx, chunk)));
                idx += 1;
            }
            None => {
                saw_non_chunk = true;
            }
        }
    }

    assert!(saw_non_chunk, "stream must contain at least one non-chunk item (e.g. State)");

    // Flush remaining buffer
    match buf {
        (Some((pp_idx, pp)), Some((p_idx, p))) => {
            on_second_to_last(pp_idx, &pp);
            on_last(p_idx, &p);
        }
        (None, Some((p_idx, p))) => {
            on_last(p_idx, &p);
        }
        _ => panic!("stream must produce at least one chunk"),
    }

    agg.expect("stream must produce at least one chunk")
}

/// Like [`consume_stream`], but with an accumulator that is built up on
/// every chunk and passed to `on_last` for richer assertion messages.
///
/// `accumulate` is called on every chunk (including second-to-last and last)
/// and can push data into `acc`.  `on_last` receives a reference to the
/// final accumulator alongside the chunk index and chunk.
pub(crate) async fn consume_stream_acc<C, S, A>(
    stream: S,
    mut push: impl FnMut(&mut C, &C),
    mut accumulate: impl FnMut(&C, &mut A),
    mut on_chunk: impl FnMut(usize, &C),
    mut on_second_to_last: impl FnMut(usize, &C),
    mut on_last: impl FnMut(usize, &C, &A),
    mut acc: A,
) -> C
where
    C: Clone,
    S: futures::Stream<Item = C> + Unpin,
{
    futures::pin_mut!(stream);

    let mut agg: Option<C> = None;
    let mut buf: (Option<(usize, C)>, Option<(usize, C)>) = (None, None);
    let mut idx: usize = 0;

    while let Some(chunk) = stream.next().await {
        // Accumulate aggregate
        match &mut agg {
            Some(a) => push(a, &chunk),
            None => agg = Some(chunk.clone()),
        }

        // Accumulate into acc
        accumulate(&chunk, &mut acc);

        // Shift buffer — flush prev_prev through on_chunk
        if let (Some((pp_idx, pp)), _) = &buf {
            on_chunk(*pp_idx, pp);
        }
        buf = (buf.1.take(), Some((idx, chunk)));
        idx += 1;
    }

    // Flush remaining buffer
    match buf {
        (Some((pp_idx, pp)), Some((p_idx, p))) => {
            on_second_to_last(pp_idx, &pp);
            on_last(p_idx, &p, &acc);
        }
        (None, Some((p_idx, p))) => {
            on_last(p_idx, &p, &acc);
        }
        _ => panic!("stream must produce at least one chunk"),
    }

    agg.expect("stream must produce at least one chunk")
}

/// Shared snapshot assertion.
///
/// When `env_var` is set to `"1"`, writes `json` to `path` (update mode).
/// Otherwise asserts that `json` matches `expected`.
pub(crate) fn assert_snapshot(json: &str, path: &str, expected: &str, env_var: &str) {
    if std::env::var(env_var).as_deref() == Ok("1") {
        std::fs::write(path, json).unwrap();
        eprintln!("Updated snapshot: {path}");
        let written = std::fs::read_to_string(path).unwrap();
        assert_eq!(json, written.trim_end());
    } else {
        assert_eq!(json, expected.trim_end());
    }
}
