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
    nick VARCHAR(32) NOT NULL,
    birth_date DATE NOT NULL,
    stack VARCHAR(32)[],
    search TSVECTOR GENERATED ALWAYS AS (
        TO_TSVECTOR('person', name || '' || nick || '' ||  ARRAY_TO_STRING_IMMUTABLE(stack, ' '))
    ) STORED,
    CONSTRAINT unique_nick UNIQUE (nick)
);

CREATE INDEX person_search_index ON person USING GIN (search);