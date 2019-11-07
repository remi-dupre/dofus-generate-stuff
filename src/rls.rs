use std::convert::TryInto;

use rand;

pub fn rls<T, R, E, W>(init: T, steps: u32, rng: &mut R, eval: E, walk: W) -> T
where
    T: Clone + Sized,
    R: rand::Rng,
    E: Fn(&T) -> f64,
    W: Fn(T, &mut R) -> T,
{
    // lambda ** (0.75 * steps)  ==  1e-3
    //   <==> (0.75 * steps) * ln(lambda)  ==  ln(1e-3)
    //   <==> ln(lambda)  ==  ln(1e-3) / (0.75 * steps)
    //   <==> lambda  ==  exp(ln(1e-3) / (0.75 * steps))
    let lambda = (1e-3f64.ln() / (0.75 * f64::from(steps))).exp();
    let init_eval = eval(&init);

    let (ret, _eval) = (1..=steps)
        .map(|step| {
            lambda.powi(step.try_into().expect("`step` must fit in a i32"))
        })
        .fold((init, init_eval), |(curr, curr_eval), threshold| {
            let new = walk(curr.clone(), rng);
            let new_eval = eval(&new);

            if new_eval > (1. - threshold) * curr_eval {
                (new, new_eval)
            } else {
                (curr, curr_eval)
            }
        });

    ret
}
