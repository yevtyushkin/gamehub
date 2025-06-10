create table player
(
    id          uuid primary key,
    screen_name varchar(30) not null,
    joined_at   timestamptz not null
);

create type third_party_sign_in_provider as enum ('Google');

create table third_party_sign_in_method
(
    provider  third_party_sign_in_provider not null,
    user_id   text                         not null,
    primary key (provider, user_id),
    player_id uuid                         not null references player (id)
);
