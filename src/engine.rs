use crate::id::Id;
use crate::prop::Prop;
use crate::tag::Tag;
use crate::task::Task;

#[derive(Debug)]
pub enum Mutation {
    SetProp(Prop),
    SetTag(Tag),
}

#[derive(Debug)]
pub enum Query {
    Id(Id),
    Tag(Tag),
}

pub type Mutations = Vec<Mutation>;
pub type Queries = Vec<Query>;

#[derive(Debug)]
pub enum CortexEngine {
    Create(Mutations),
    Read(Queries),
    Update(Queries, Mutations),
    Delete(Queries),
}

impl CortexEngine {
    pub fn run(
        self,
        input_tasks_iter: impl Iterator<Item = Result<Task, String>>,
        put_task: impl Fn(&Task) -> Result<(), String>,
    ) -> Vec<Task> {
        match &self {
            CortexEngine::Create(mutations) => {
                let mut new_task = Task::generate();

                new_task.apply_mutations(mutations);

                put_task(&new_task);

                vec![new_task]
            }

            CortexEngine::Read(queries) => input_tasks_iter
                .map( |r| r.unwrap() )
                .filter(|t| t.satisfies_queries(queries))
                .collect::<Vec<Task>>(),

            _ => vec![],
        }
    }
}
