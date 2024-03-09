CREATE EXTENSION unaccent;

ALTER TEXT SEARCH DICTIONARY unaccent (RULES = 'unaccent');

CREATE TEXT SEARCH CONFIGURATION person (COPY = portuguese);

ALTER TEXT SEARCH CONFIGURATION person ALTER MAPPING FOR hword, hword_part, word WITH unaccent, portuguese_stem;

CREATE OR REPLACE FUNCTION ARRAY_TO_STRING_IMMUTABLE (
 arr TEXT[],
 sep TEXT
) RETURNS TEXT IMMUTABLE PARALLEL SAFE LANGUAGE SQL AS $$
SELECT ARRAY_TO_STRING(arr, sep) $$;

CREATE TABLE person (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    nick VARCHAR(32) CONSTRAINT ID_PK PRIMARY KEY,
    birth_date DATE NOT NULL,
    stack VARCHAR(32)[],
    BUSCA_TRGM TEXT GENERATED ALWAYS AS (
        LOWER(NOME || APELIDO || STACK)
    ) STORED
);

CREATE EXTENSION PG_TRGM;
CREATE INDEX CONCURRENTLY IF NOT EXISTS IDX_PESSOAS_BUSCA_TGRM ON person USING GIST (BUSCA_TRGM GIST_TRGM_OPS(SIGLEN=64));