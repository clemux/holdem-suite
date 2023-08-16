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
}

export type Seat = {
    hand_id: string,
    player_name: string,
    seat_number: number,
    stack: number,
    bounty: number | null,
}

export type Action = {
    id: number,
    hand_id: string,
    player_name: string,
    street: string,
    action_type: string,
    amount: number,
    is_allin: boolean,
}