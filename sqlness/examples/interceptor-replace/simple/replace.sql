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
