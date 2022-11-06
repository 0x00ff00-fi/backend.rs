INSERT INTO public.posts(name, icon, content, media)
VALUES ($1, $2, $3, $4)
RETURNING $table_fields;
