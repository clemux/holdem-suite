export type Table = {
  name: string,
  rs_table: object,
}

export type Player = {
  name: string,
  nb_hands: number,
}

export type PlayerStats = {
    vpip: number,
    pfr: number,
    three_bet: number,
}

export type Hand = {
    id: string,
    hole_card_1: string,
    hole_card_2: string,
    tournament_id: number,
    cash_game_name: string,
    datetime: string,
}

export type Seat = {
    hand_id: string,
    player_name: string,
    seat_number: number,
    stack: number,
    bounty: number|null,
}