use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::cooldown::Cooldown;

pub enum VoteError {
    Type,
    Progress,
    Nomination,
    Voters,
    Cooldown(f32),
}

pub enum VoteResult {
    None,
    Yay(String, String),
    Nay,
}

pub struct Ballot {
    voting: bool,
    voting_duration: Cooldown,
    voters: i8,
    yays: i8,
    nays: i8,
    player_vote: HashMap<String, bool>,
    player_cooldown: HashMap<String, Cooldown>,
    cooldown_duration: Duration,
    target: f32,
    nominations: HashMap<String, HashSet<String>>,
    proposed: String,
    r#type: String,
}

impl Ballot {
    pub fn new(
        voting_duration: Duration,
        cooldown_duration: Duration,
        target: f32,
        nominations: HashMap<String, HashSet<String>>,
    ) -> Ballot {
        Ballot {
            voting: false,
            voting_duration: Cooldown::new(voting_duration),
            voters: 0,
            yays: 0,
            nays: 0,
            player_vote: HashMap::new(),
            player_cooldown: HashMap::new(),
            cooldown_duration: cooldown_duration + voting_duration,
            target: target,
            nominations: nominations,
            proposed: String::new(),
            r#type: String::new(),
        }
    }
    fn reset(&mut self) {
        self.voting = false;
        self.yays = 0;
        self.nays = 0;
        self.voting_duration.clear_cooldown();
        self.player_vote.clear();
    }
    pub fn start_voting(
        &mut self,
        id: &String,
        r#type: &String,
        proposal: &String,
    ) -> Result<(), VoteError> {
        if self.is_user_in_cooldown(id) {
            return Err(VoteError::Cooldown(self.get_user_cooldown(id)));
        }
        if self.voting {
            return Err(VoteError::Progress);
        }
        if !self.nominations.contains_key(r#type) {
            return Err(VoteError::Type);
        }
        if !self.nominations.get(r#type).unwrap().contains(proposal) {
            return Err(VoteError::Nomination);
        }
        self.put_user_in_cooldown(id);
        self.voting_duration.put_in_cooldown();
        self.voting = true;
        self.proposed = proposal.to_string();
        self.r#type = r#type.to_string();
        Ok(())
    }
    pub fn stop_voting(&mut self) -> Result<(), VoteError> {
        if !self.voting {
            return Err(VoteError::Progress);
        }
        self.reset();
        Ok(())
    }
    pub fn vote(&mut self, id: &String, vote: bool) -> Result<(), VoteError> {
        self.unvote(id)?;
        self.player_vote.insert(id.to_owned(), vote);
        match vote {
            true => self.yays += 1,
            false => self.nays += 1,
        }
        Ok(())
    }
    pub fn unvote(&mut self, id: &String) -> Result<(), VoteError> {
        if !self.voting {
            return Err(VoteError::Progress);
        }
        match self.player_vote.remove(id) {
            Some(value) => match value {
                true => self.yays -= 1,
                false => self.nays -= 1,
            },
            None => {}
        }
        Ok(())
    }
    pub fn set_voters(&mut self, voters: i8) -> Result<(), VoteError> {
        if !self.voting {
            return Err(VoteError::Progress);
        }
        self.voters = voters;
        Ok(())
    }
    pub fn increment_voters(&mut self) -> Result<(), VoteError> {
        self.set_voters(self.voters + 1)
    }
    pub fn decrement_voters(&mut self) -> Result<(), VoteError> {
        self.set_voters(self.voters - 1)
    }
    pub fn get_requirements(&self) -> (i8, i8) {
        let voters_yay = (self.voters as f32 * self.target).ceil() as i8;
        let voters_nay = self.voters - voters_yay;
        (voters_yay, if voters_nay == 0 { 1 } else { voters_nay })
    }
    pub fn get_result(
        &self,
        ignore_cooldown: bool,
        majority_result: bool,
    ) -> Result<VoteResult, VoteError> {
        if !self.voting {
            return Err(VoteError::Progress);
        }
        if self.voters == 0 {
            return Err(VoteError::Voters);
        }
        if self.voting_duration.is_in_cooldown() && !ignore_cooldown {
            return Err(VoteError::Cooldown(
                self.voting_duration.get_remaining_time(),
            ));
        }
        let (yay, nay) = self.get_requirements();
        match majority_result {
            true => {
                if self.yays > self.nays {
                    Ok(VoteResult::Yay(self.r#type.clone(), self.proposed.clone()))
                } else if self.nays > self.yays {
                    Ok(VoteResult::Nay)
                } else {
                    Err(VoteError::Progress)
                }
            }
            false => {
                if self.yays > yay {
                    Ok(VoteResult::Yay(self.r#type.clone(), self.proposed.clone()))
                } else if self.nays > nay {
                    Ok(VoteResult::Nay)
                } else {
                    Ok(VoteResult::None)
                }
            }
        }
    }
    pub fn get_votes(&self) -> (i8, i8) {
        (self.yays, self.nays)
    }
    pub fn is_voting_finished(&self) -> bool {
        self.voting_duration.is_in_cooldown()
    }
    pub fn is_user_in_cooldown(&mut self, id: &String) -> bool {
        match self.player_cooldown.get(id) {
            Some(value) => value.is_in_cooldown(),
            None => {
                self.player_cooldown
                    .insert(id.to_owned(), Cooldown::new(self.cooldown_duration));
                false
            }
        }
    }
    pub fn get_user_cooldown(&mut self, id: &String) -> f32 {
        match self.player_cooldown.get(id) {
            Some(value) => value.get_remaining_time(),
            None => 0f32,
        }
    }
    pub fn put_user_in_cooldown(&mut self, id: &String) {
        match self.player_cooldown.get_mut(id) {
            Some(value) => {
                value.put_in_cooldown();
            }
            None => {
                let mut cooldown = Cooldown::new(self.cooldown_duration);
                cooldown.put_in_cooldown();
                self.player_cooldown.insert(id.to_owned(), cooldown);
            }
        }
    }
    pub fn remove_user_cooldown(&mut self, id: &String) {
        self.player_cooldown.remove(id);
    }
    pub fn get_type(&self) -> String {
        self.r#type.clone()
    }
}
