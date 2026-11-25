pub mod in_memory;
pub mod postgres;

pub use in_memory::InMemoryInstitutionRepository;
pub use postgres::PostgresInstitutionRepository;

#[cfg(test)]
pub mod test_fixtures;
