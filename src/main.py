#!/usr/bin/env python3

from fastapi import FastAPI
from fastapi.responses import RedirectResponse
from kry import DEV_ENV, Base
from typing import Union

APP_NAME = "Links"
if DEV_ENV:
    app = FastAPI(debug=True, title=APP_NAME)
else:
    app = FastAPI(openapi_url=None, redoc_url=None, title=APP_NAME)


@app.get("/list")
def get_links_list():
    """Redirect back to deta page"""
    return RedirectResponse(Base("others").get("deta_page")["value"])


ERR_MSG = dict(detail="Link Not Found")


RelU = lambda x: 0 if x < 0 else x


@app.get("/{name}/{num}")
def redirect_numbered_link(name: str, num: Union[int, float]):
    """Redirecting Numbered Links"""
    link_obj = Base("links").get(name)
    if not link_obj:
        return ERR_MSG
    link: str = link_obj["link"]
    prefix_zeros: int = link_obj["prefix_zeros"]
    return RedirectResponse(
        link.format(RelU(prefix_zeros - len(str(int(num)))) * "0" + str(num))
    )


@app.get("/{name}")
def redirect_link(name: str):
    """Redirecting  Links"""
    link_obj = Base("links").get(name)
    if not link_obj:
        return ERR_MSG
    link: str = link_obj["link"]
    if "{0}" in link:
        return ERR_MSG
    return RedirectResponse(link)


if DEV_ENV:
    import uvicorn

    if __name__ == "__main__":
        uvicorn.run("main:app", host="0.0.0.0", port=3030, reload=True)
