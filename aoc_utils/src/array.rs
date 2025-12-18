use std::mem::MaybeUninit;

pub fn try_from_fn<T, E, const N: usize>(
    mut f: impl FnMut(usize) -> Result<T, E>,
) -> Result<[T; N], E> {
    let mut array: [_; N] = std::array::from_fn(|_| MaybeUninit::uninit());
    for (i, elem) in array.iter_mut().enumerate() {
        *elem = MaybeUninit::new(f(i)?);
    }
    Ok(array.map(|x| unsafe { x.assume_init() }))
}

pub fn from_iter<T, const N: usize>(iter: impl IntoIterator<Item = T>) -> Result<[T; N], Vec<T>> {
    let mut n = 0;
    let mut fused = iter.into_iter().fuse();
    let array: [_; N] = std::array::from_fn(|i| {
        if let Some(elem) = fused.next() {
            n = i + 1;
            MaybeUninit::new(elem)
        } else {
            MaybeUninit::uninit()
        }
    });

    if n != N {
        // received fewer elements from the iterator than N
        Err(array
            .into_iter()
            .take(n)
            .map(|x| unsafe { x.assume_init() })
            .collect())
    } else {
        Ok(array.map(|x| unsafe { x.assume_init() }))
    }
}

pub fn from_iter_exact<T, const N: usize>(
    iter: impl IntoIterator<Item = T>,
) -> Result<[T; N], Vec<T>> {
    let mut n = 0;
    let mut fused = iter.into_iter().fuse();
    let array: [_; N] = std::array::from_fn(|i| {
        if let Some(elem) = fused.next() {
            n = i + 1;
            MaybeUninit::new(elem)
        } else {
            MaybeUninit::uninit()
        }
    });

    if n != N {
        // received fewer elements from the iterator than N
        Err(array
            .into_iter()
            .take(n)
            .map(|x| unsafe { x.assume_init() })
            .collect())
    } else if let Some(extra) = fused.next() {
        // received at least one more element than expected
        let mut vec: Vec<T> = array
            .into_iter()
            .map(|x| unsafe { x.assume_init() })
            .collect();
        vec.push(extra);
        vec.extend(fused);
        Err(vec)
    } else {
        Ok(array.map(|x| unsafe { x.assume_init() }))
    }
}

pub fn try_from_iter<T, E, const N: usize>(
    iter: impl IntoIterator<Item = Result<T, E>>,
) -> Result<Result<[T; N], Vec<T>>, E> {
    let mut n = 0;
    let mut fused = iter.into_iter().fuse();
    let array: [_; N] = try_from_fn(|i| {
        if let Some(elem) = fused.next() {
            n = i + 1;
            Ok(MaybeUninit::new(elem?))
        } else {
            Ok(MaybeUninit::uninit())
        }
    })?;

    Ok(if n != N {
        // received fewer elements from the iterator than N
        Err(array
            .into_iter()
            .take(n)
            .map(|x| unsafe { x.assume_init() })
            .collect())
    } else {
        Ok(array.map(|x| unsafe { x.assume_init() }))
    })
}

pub fn try_from_iter_exact<T, E, const N: usize>(
    iter: impl IntoIterator<Item = Result<T, E>>,
) -> Result<Result<[T; N], Vec<T>>, E> {
    let mut n = 0;
    let mut fused = iter.into_iter().fuse();
    let array: [_; N] = try_from_fn(|i| {
        if let Some(elem) = fused.next() {
            n = i + 1;
            Ok(MaybeUninit::new(elem?))
        } else {
            Ok(MaybeUninit::uninit())
        }
    })?;

    Ok(if n != N {
        // received fewer elements from the iterator than N
        Err(array
            .into_iter()
            .take(n)
            .map(|x| unsafe { x.assume_init() })
            .collect())
    } else if let Some(extra) = fused.next() {
        // received at least one more element than expected
        let mut vec: Vec<T> = array
            .into_iter()
            .map(|x| unsafe { x.assume_init() })
            .collect();
        vec.push(extra?);
        for extra in fused {
            vec.push(extra?);
        }
        Err(vec)
    } else {
        Ok(array.map(|x| unsafe { x.assume_init() }))
    })
}

pub fn try_map<T, U, E, const N: usize>(
    array: [T; N],
    f: impl FnMut(T) -> Result<U, E>,
) -> Result<[U; N], E> {
    match try_from_iter(array.into_iter().map(f)) {
        Ok(Ok(arr)) => Ok(arr),
        Err(err) => Err(err),
        Ok(Err(_)) => unreachable!("array size must be the same"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use std::num::{IntErrorKind, ParseIntError};
    use std::str::FromStr;

    fn unfused_iter() -> impl Iterator<Item = i32> {
        let mut n = 0;
        std::iter::from_fn(move || {
            n += 1;
            if n > 1 { Some(n) } else { None }
        })
    }

    #[test]
    fn try_from_fn_empty() {
        assert_eq!(try_from_fn(|_| "".parse::<i32>()), Ok([]));
    }

    #[test]
    fn try_from_fn_all_ok() {
        let mut iter = [1, 2, 3].into_iter();
        assert_eq!(try_from_fn(move |_| iter.next().ok_or(())), Ok([1, 2, 3]));
    }

    #[test]
    fn try_from_fn_single_error() {
        let mut iter = ["1", "two", "3"].into_iter();
        assert_matches!(
            try_from_fn(move |_| iter.next().unwrap_or("").parse()),
            Err::<[i32; 3], ParseIntError>(ParseIntError { .. })
        );
    }

    #[test]
    fn try_from_fn_calls_from_index() {
        assert_eq!(try_from_fn(Ok::<usize, ()>), Ok([0, 1, 2, 3]));
    }

    #[test]
    fn try_from_fn_short_circuits() {
        let _: Result<[(); 3], ()> = try_from_fn(|i| match i {
            0 => Ok(()),
            1 => Err(()),
            _ => panic!("mapping function should not have been evaluated for {i}"),
        });
    }

    #[test]
    fn from_iter_empty() {
        assert_eq!(from_iter(std::iter::empty::<i32>()), Ok([]))
    }

    #[test]
    fn from_iter_with_correct_number_of_elements() {
        assert_eq!(from_iter(1..=3), Ok([1, 2, 3]));
    }

    #[test]
    fn from_iter_with_extra_elements() {
        assert_eq!(from_iter(1..), Ok([1, 2, 3]));
    }

    #[test]
    fn from_iter_with_too_few_elements() {
        assert_eq!(from_iter(1..=3), Err::<[i32; 4], Vec<i32>>(vec![1, 2, 3]));
    }

    #[test]
    fn from_iter_with_unfused_iter() {
        assert_eq!(from_iter(unfused_iter()), Err::<[i32; 3], Vec<i32>>(vec![]));
    }

    #[test]
    fn from_iter_exact_with_correct_number_of_elements() {
        assert_eq!(from_iter_exact(1..=3), Ok([1, 2, 3]));
    }

    #[test]
    fn from_iter_exact_with_extra_elements() {
        assert_eq!(
            from_iter_exact(1..=5),
            Err::<[i32; 3], Vec<i32>>(vec![1, 2, 3, 4, 5])
        );
    }

    #[test]
    fn from_iter_exact_with_too_few_elements() {
        assert_eq!(
            from_iter_exact(1..=3),
            Err::<[i32; 4], Vec<i32>>(vec![1, 2, 3])
        );
    }

    #[test]
    fn from_iter_exact_with_unfused_iter() {
        assert_eq!(
            from_iter_exact(unfused_iter().take(3)),
            Err::<[i32; 3], Vec<i32>>(vec![])
        );
    }

    #[test]
    fn try_from_iter_with_correct_number_of_ok_elements() {
        assert_eq!(
            try_from_iter(["1", "2", "3"].map(i32::from_str)),
            Ok(Ok([1, 2, 3]))
        );
    }

    #[test]
    fn try_from_iter_with_too_few_ok_elements() {
        assert_eq!(
            try_from_iter(["1", "2", "3"].map(i32::from_str)),
            Ok(Err::<[i32; 4], Vec<i32>>(vec![1, 2, 3]))
        );
    }

    #[test]
    fn try_from_iter_with_extra_ok_elements() {
        assert_eq!(
            try_from_iter(["1", "2", "3"].map(i32::from_str)),
            Ok(Ok([1, 2]))
        );
    }

    #[test]
    fn try_from_iter_with_unfused_iter() {
        assert_eq!(
            try_from_iter(unfused_iter().map(Ok::<_, ParseIntError>)),
            Ok(Err::<[i32; 3], Vec<i32>>(vec![]))
        );
    }

    #[test]
    fn try_from_iter_with_single_error_in_range() {
        assert_matches!(
            try_from_iter(["1", "two", "3"].map(i32::from_str)),
            Err::<Result<[i32; 4], _>, _>(ParseIntError { .. })
        );
    }

    #[test]
    fn try_from_iter_with_single_error_outside_range() {
        assert_eq!(
            try_from_iter(["1", "2", "three"].map(i32::from_str)),
            Ok(Ok([1, 2]))
        );
    }

    #[test]
    fn try_map_empty() {
        assert_eq!(try_map([], i32::from_str), Ok([]));
    }

    #[test]
    fn try_map_all_ok() {
        assert_eq!(try_map(["1", "2", "3"], i32::from_str), Ok([1, 2, 3]));
    }

    #[test]
    fn try_map_single_error() {
        assert_matches!(
            try_map(["1", "two", "3"], i32::from_str),
            Err(ParseIntError { .. })
        );
    }

    #[test]
    fn try_map_first_error_wins() {
        assert_matches!(
            try_map(["1", "200", "-300"], i8::from_str)
                .unwrap_err()
                .kind(),
            IntErrorKind::PosOverflow
        );
    }

    #[test]
    fn try_map_short_circuits() {
        try_map([1, 2, 3], |i| match i {
            1 => Ok(()),
            2 => Err(()),
            _ => panic!("mapping function should not have been evaluated for {i}"),
        })
        .unwrap_err();
    }
}
