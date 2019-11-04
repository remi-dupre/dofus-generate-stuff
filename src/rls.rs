use rand;
use std::cmp;

pub trait Blackbox: std::marker::Sized + Clone {
    fn eval(&self) -> f64;

    fn bb_find_max<R>(
        init: Self,
        steps: u64,
        rng: &mut R,
        walk: impl Fn(Self, &mut R) -> Self,
    ) -> Self
    where
        R: rand::Rng,
    {
        let init_eval = init.eval();
        let (ret, _eval) = (1..=steps)
            .map(|step| cmp::min(steps, step + step / 5) as f64 / steps as f64)
            .fold((init, init_eval), |(curr, curr_eval), threshold| {
                let new = walk(curr.clone(), rng);
                let new_eval = new.eval();

                if new_eval >= threshold * curr_eval {
                    (new, new_eval)
                } else {
                    (curr, curr_eval)
                }
            });
        ret
    }
}
