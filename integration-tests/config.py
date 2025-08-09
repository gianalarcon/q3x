from uuid import uuid4

from ic import Principal
from ic.canister import Canister
from ic.client import Client
from ic.identity import Identity
from ic.agent import Agent
from mnemonic import Mnemonic


def get_wallet_id():
    return f'test_{str(uuid4())[0:8]}'


def assert_ok(res):
    assert 'Ok' in res[0], f'Result not ok: {res}'


def assert_err(res, error=""):
    assert 'Err' in res[0], 'Result not err'
    if error:
        assert res[0]['Err'] == error, 'Error not match'


def get_id(container):
    return "bkyz2-fmaaa-aaaaa-qaaaq-cai"


def get_default_identities():
    return [
        Identity.from_seed("dumb crucial heart army senior rubber tomorrow uncover brown upgrade road start"),
        Identity.from_seed("sad tiger kite quote erupt auction apple sight barely utility adult reason"),
        Identity.from_seed("turkey enroll pride credit mistake toast speak million report phrase eye margin")
    ]


def get_default_principals():
    return [_.sender().to_str() for _ in get_default_identities()]


def get_agent():
    return Agent(
        get_default_identities()[0], Client(url="http://127.0.0.1:4943")
    )


def create_safe():
    return Canister(
        agent=get_agent(),
        canister_id=get_id("q3x_backend"),
        candid=open("./src/q3x_backend/q3x_backend.did").read()
    )
