use sampling::{HasRng, MarkovChain};
use rand::Rng;


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// # Result of flipping a coin
pub enum CoinFlip {
    /// The result is Head
    Head,
    /// The result is Tail
    Tail
}

impl CoinFlip
{
    /// Turn Coin around, i.e., invert CoinFlip
    pub fn turn(&mut self) {
        *self = match self {
            CoinFlip::Head => CoinFlip::Tail,
            CoinFlip::Tail => CoinFlip::Head
        };
    }
}

#[derive(Clone, Debug)]
/// Result of markov Step
pub struct CoinFlipMove{
    previous: CoinFlip,
    index: usize,
}

#[derive(Clone, Debug)]
/// # A sequence of Coin flips. Contains random Number generator
pub struct CoinFlipSequence<R> {
    rng: R,
    seq: Vec<CoinFlip>,
    /// You can ignore everything after here, it is just used for testing
    steps: usize,
    rejected: usize,
    accepted: usize,
    undo_count: usize
}


impl<R> CoinFlipSequence<R>
    where R: Rng,
{
    /// Create new coin flip sequence
    /// * length `n`
    /// * use `rng` as random number generator
    pub fn new(
        n: usize, 
        mut rng: R
    ) -> Self
    {
        let mut seq = Vec::with_capacity(n);
        seq.extend(
            (0..n).map(|_| {
                if rng.gen::<bool>() {
                    CoinFlip::Tail
                } else {
                    CoinFlip::Head
                }
            })
        );
        Self{
            rng,
            seq,
            steps:0,
            rejected: 0,
            accepted: 0,
            undo_count: 0
        }
    }
}


impl<R> CoinFlipSequence<R>
{
    /// Count how often `Head` occurs in the Coin flip sequence
    pub fn head_count(&self) -> u32
    {
        let mut head_count = 0;
        self.seq.iter()
            .filter(|&item| *item == CoinFlip::Head)
            .for_each(|_| head_count += 1);
        head_count
    }

    /// * Calculate the head count, if a previouse head count of the ensemble and the 
    ///     markov steps leading to the current state are known
    /// * `head_count` is updated
    /// * might **panic** if `step` was not the markov step leading from the ensemble with `head_count`
    ///     to the current ensemble - if it does not panic, the result will be wrong
    pub fn update_head_count(&self, step: &CoinFlipMove, head_count: &mut u32)
    {
        match step.previous {
            CoinFlip::Head => {
                *head_count -= 1;
            },
            CoinFlip::Tail => {
                *head_count += 1;
            }
        }
    }
}

impl<R> MarkovChain<CoinFlipMove, ()> for CoinFlipSequence<R>
where R: Rng
{
    /// Perform a markov step
    fn m_step(&mut self) -> CoinFlipMove {
        // draw a random position
        let pos = self.rng.gen_range(0..self.seq.len());
        let previous = self.seq[pos];
        // flip coin at that position
        self.seq[pos].turn();
        // information to restore the previouse state
        CoinFlipMove{
            previous,
            index: pos
        }
    }

    /// # Only implemented for testcases
    /// Default implementation would suffice
    #[inline]
    fn m_steps(&mut self, count: usize, steps: &mut Vec<CoinFlipMove>) {
        self.steps += 1;
        steps.clear();
        steps.extend((0..count)
            .map(|_| self.m_step())
        );
    }

    /// # Only implemented for testcases
    /// Default implementation would suffice
    #[inline]
    fn m_steps_acc<Acc, AccFn>
    (
        &mut self,
        count: usize,
        steps: &mut Vec<CoinFlipMove>,
        acc: &mut Acc,
        mut acc_fn: AccFn
    )
    where AccFn: FnMut(&Self, &CoinFlipMove, &mut Acc)
    {
        self.steps += 1;
        steps.clear();
        steps.extend(
            (0..count)
                .map(|_| self.m_step_acc(acc, &mut acc_fn))
        );
    }

    fn undo_step(&mut self, step: &CoinFlipMove) {
        self.seq[step.index] = step.previous;
    }

    #[inline]
    fn undo_step_quiet(&mut self, step: &CoinFlipMove) {
        self.undo_step(step);   
    }

    /// # Only implemented for testcases
    /// Default implementation would suffice
    fn undo_steps(&mut self, steps: &[CoinFlipMove], res: &mut Vec<()>) {
        self.undo_count += 1;
        res.clear();
        res.extend(
            steps.iter()
                .rev()
                .map(|step| self.undo_step(step))
        );
        assert_eq!(self.rejected, self.undo_count);
    }

    /// # Only implemented for testcases
    /// Default implementation would suffice
    fn undo_steps_quiet(&mut self, steps: &[CoinFlipMove]) {
        self.undo_count += 1;
        steps.iter()
            .rev()
            .for_each( |step| self.undo_step_quiet(step));
        assert_eq!(self.rejected, self.undo_count);
    }

    /// # Only implemented for testcases
    /// Default implementation would suffice
    fn steps_accepted(&mut self, _steps: &[CoinFlipMove])
    {
        self.accepted += 1;
        if self.accepted + self.rejected != self.steps{
            panic!("{} {} {}", self.steps, self.rejected, self.accepted)
        }
    }

    /// # Only implemented for testcases
    /// Default implementation would suffice
    fn steps_rejected(&mut self, _steps: &[CoinFlipMove])
    {
        self.rejected += 1;
        if self.accepted + self.rejected != self.steps{
            panic!("{} {} {}", self.steps, self.rejected, self.accepted)
        }

    }
}

impl<R> HasRng<R> for CoinFlipSequence<R>
    where R: Rng
{
    fn rng(&mut self) -> &mut R {
        &mut self.rng
    }

    fn swap_rng(&mut self, rng: &mut R) {
        std::mem::swap(&mut self.rng, rng);
    }
}