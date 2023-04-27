-- SQLNESS ENV SECRET
select {{ SECRET }} from data;

-- SQLNESS ENV SECRET NONEXISTENT
select {{ SECRET }} from data;