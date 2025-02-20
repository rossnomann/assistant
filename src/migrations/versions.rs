use barrel::{Migration, backend::Pg, types};

macro_rules! version {
    ($builder:ident) => {
        Version::new(stringify!($builder), $builder)
    };
}

pub fn build() -> Vec<Version> {
    vec![version!(create_notes)]
}

pub struct Version {
    name: String,
    builder: Builder,
}

impl Version {
    fn new<N>(name: N, builder: Builder) -> Self
    where
        N: Into<String>,
    {
        Self {
            name: name.into(),
            builder,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn build(&self) -> String {
        let builder = self.builder;
        let migration = builder();
        migration.make::<Pg>()
    }
}

type Builder = fn() -> Migration;

fn create_notes() -> Migration {
    let mut migration = Migration::new();
    migration.create_table("notes", |table| {
        table.add_column("id", types::primary());
        table.add_column("keywords", types::array(&types::varchar(255)));
        table.add_column("data", types::json());
    });
    migration
}
