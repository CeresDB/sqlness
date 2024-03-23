-- SQLNESS ARG bla=a addr=127.0.0.1 port=3306
SELECT * FROM t;

-- SQLNESS   ARG port=3307
SELECT *
FROM t
WHERE A=B;

-- multiple env in one line
-- SQLNESS ENV ENV1 ENV2 NONEXISTENT1 NONEXISTENT2 NONEXISTENT3
SELECT $ENV1, $ENV2, $NONEXISTENT1 FROM t;

-- multiple env in multiple lines
-- SQLNESS ENV ENV1
-- SQLNESS ENV ENV2
-- SQLNESS ENV NONEXISTENT1
-- SQLNESS ENV NONEXISTENT2
-- SQLNESS ENV NONEXISTENT3
SELECT $ENV1, $ENV2, $NONEXISTENT1, FROM t;

-- Undeclared env won't be rendered
-- SQLNESS ENV ENV2
SELECT $ENV1, $ENV2, $NONEXISTENT1 FROM t;

-- SQLNESS REPLACE 00
SELECT 0;

-- SQLNESS REPLACE 00
SELECT 00;

-- SQLNESS REPLACE 0 1
SELECT 0;

-- example of capture group replacement
-- SQLNESS REPLACE (?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2}) $m/$d/$y
2012-03-14, 2013-01-01 and 2014-07-05;

-- SQLNESS TEMPLATE {"name": "test"}
SELECT * FROM table where name = "{{name}}";

-- SQLNESS TEMPLATE {"aggr": ["sum", "avg", "count"]}
{% for item in aggr %}
SELECT {{item}}(c) from t {%if not loop.last %} {{sql_delimiter()}} {% endif %}
{% endfor %}
;

-- SQLNESS TEMPLATE
INSERT INTO t (c) VALUES
{% for num in range(1, 5) %}
({{ num }}) {%if not loop.last %} , {% endif %}
{% endfor %}
;

-- SQLNESS SORT_RESULT
4
3
6
1;

-- SQLNESS SORT_RESULT 1 1
7
1
4
2
2;
