import os
from deta import Deta

DEV_ENV: bool = (
    (os.getenv("DETA_SPACE_APP") and os.getenv("DETA_SPACE_APP") != "true")  # type: ignore
    or (os.getenv("VIRTUAL_ENV") is not None)
    or (os.getenv("PIPENV_ACTIVE") and os.getenv("PIPENV_ACTIVE") == "1")
)
PROD_ENV: bool = not DEV_ENV
DEV_ENV = not PROD_ENV

KEY = ""
if DEV_ENV:
    import dotenv

    dotenv.load_dotenv()
    KEY = os.getenv("DETA_KEY")
else:
    KEY = os.getenv("DETA_PROJECT_KEY")

deta = Deta(str(KEY))

Base = deta.Base
