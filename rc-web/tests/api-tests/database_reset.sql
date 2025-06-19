drop table definitions cascade;

drop table event cascade;

drop table event_listener cascade;

drop table event_sequence cascade;

drop function event_store_begin_epoch ();

drop function event_store_current_epoch ();

drop function notify_event_listener ();

drop sequence event_sequence_event_id_seq cascade;

drop table client_projection cascade;
drop table student_projection cascade;
drop table students cascade;
drop table insurance_projection cascade;
drop table official_projection cascade;
drop table client_projection cascade;
drop table consultant_projection cascade;
drop table teacher_projection cascade;

\dt -- show all relations
