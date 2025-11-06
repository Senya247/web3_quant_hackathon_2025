import requests
import time
import hmac
import hashlib


BASE_URL = "https://mock-api.roostoo.com"
API_KEY = "bmpkW4mdv3ph7q3I9lHAsDCknhrn4nemJI9e50F0iluZkLBqGznQB8y9TWHQCFcK"
SECRET_KEY = "FerUSsHuulHOB3b6e6HhCeSy6xXmYMKjh1b9Kj0U4HFiz3uTr4kuqALHmFQXrbgB"


def _get_timestamp():
    """Returns a 13-digit millisecond timestamp as a string."""
    return str(int(time.time() * 1000))


def _get_signed_headers(payload={}):
    """
    Creates a signature for a given payload (dict) and returns
    the correct headers for a SIGNED (RCL_TopLevelCheck) request.
    """
    # 1. Add timestamp to the payload
    payload["timestamp"] = _get_timestamp()

    # 2. Sort keys and create the totalParams string
    sorted_keys = sorted(payload.keys())
    total_params = "&".join(f"{key}={payload[key]}" for key in sorted_keys)

    # 3. Create HMAC-SHA256 signature
    signature = hmac.new(
        SECRET_KEY.encode("utf-8"), total_params.encode("utf-8"), hashlib.sha256
    ).hexdigest()

    # 4. Create headers
    headers = {"RST-API-KEY": API_KEY, "MSG-SIGNATURE": signature}

    return headers, payload, total_params


def get_pending_count():
    """Gets pending order count. (Auth: RCL_TopLevelCheck)"""
    url = f"{BASE_URL}/v3/pending_count"

    headers, payload, total_params_string = _get_signed_headers(payload={})

    try:
        response = requests.get(url, headers=headers, params=payload)
        response.raise_for_status()
        return response.json()
    except requests.exceptions.RequestException as e:
        print(f"Error getting pending count: {e}")
        print(f"Response text: {e.response.text if e.response else 'N/A'}")
        return None


def place_order(pair_or_coin, side, quantity, price=None, order_type=None):
    """
    Place a LIMIT or MARKET order.
    """
    url = f"{BASE_URL}/v3/place_order"
    pair = f"{pair_or_coin}/USD" if "/" not in pair_or_coin else pair_or_coin

    if order_type is None:
        order_type = "LIMIT" if price is not None else "MARKET"

    if order_type == "LIMIT" and price is None:
        print("Error: LIMIT orders require 'price'.")
        return None

    payload = {
        "pair": pair,
        "side": side.upper(),
        "type": order_type.upper(),
        "quantity": str(quantity),
    }
    if order_type == "LIMIT":
        payload["price"] = str(price)

    headers, _, total_params = _get_signed_headers(payload)
    headers["Content-Type"] = "application/x-www-form-urlencoded"

    try:
        res = requests.post(url, headers=headers, data=total_params)
        res.raise_for_status()
        return res.json()
    except requests.exceptions.RequestException as e:
        print(f"Error placing order: {e}")
        print(f"Response text: {e.response.text if e.response else 'N/A'}")
        return None


print(place_order("DOGE/USD", "buy", 7))
