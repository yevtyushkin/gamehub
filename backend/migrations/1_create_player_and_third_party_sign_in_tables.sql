create type third_party as enum('Google');

create table player(
  id uuid primary key,
  name varchar(20) not null
);

create table third_party_sign_in(
  third_party third_party not null,
  third_party_id varchar(50) not null,
  player_id uuid not null references player(id),
  primary key (third_party, third_party_id)
);
