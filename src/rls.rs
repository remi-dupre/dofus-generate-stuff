use rand;
use std::cmp;

pub fn rls<T, R, E, W>(init: T, steps: u64, rng: &mut R, eval: E, walk: W) -> T
where
    T: Clone + Sized,
    R: rand::Rng,
    E: Fn(&T) -> f64,
    W: Fn(T, &mut R) -> T,
{
    let init_eval = eval(&init);
    let (ret, _eval) = (1..=steps)
        .map(|step| cmp::min(steps, step + step / 5) as f64 / steps as f64)
        .fold((init, init_eval), |(curr, curr_eval), threshold| {
            let new = walk(curr.clone(), rng);
            let new_eval = eval(&new);

            if new_eval >= threshold * curr_eval {
                (new, new_eval)
            } else {
                (curr, curr_eval)
            }
        });
    ret
}
