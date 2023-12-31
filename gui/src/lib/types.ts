export type WindowPosition = {
    x: number,
    y: number,
    width: number,
    height: number,
}

export type Table = {
    name: string,
    rs_table: object,
    window_position: WindowPosition,
}

export type Player = {
    name: string,
}

export type PlayerStats = {
    vpip: number,
    pfr: number,
    three_bet: number,
    open_limp: number,
    nb_hands: number,
}

export type Hand = {
    id: string,
    hole_card_1: string,
    hole_card_2: string,
    tournament_id: number,
    cash_game_name: string,
    datetime: string,
    max_players: number,
    hero: string,
    button: number,
    ante: number,
    small_blind: number,
    big_blind: number,
    flop1: string,
    flop2: string,
    flop3: string,
    turn: string,
    river: string,
}

export type Seat = {
    hand_id: string,
    player_name: string,
    seat_number: number,
    stack: number,
    bounty: number | null,
    card1: string | null,
    card2: string | null,
}

export type Action = {
    id: number,
    hand_id: string,
    player_name: string,
    street: string,
    action_type: string,
    amount: number,
    is_all_in: boolean,
}

export type Card = {
    suit: string,
    rank: string,
}