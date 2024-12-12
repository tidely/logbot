#!/usr/bin/env python3

from werkzeug.security import check_password_hash
from flask import Blueprint, render_template, request, redirect, url_for, flash
from flask_login import login_user, login_required, logout_user
from .models import User

auth_blueprint = Blueprint("auth", __name__)


@auth_blueprint.route("/login", methods=["GET", "POST"])
def login():
    if request.method == "GET":
        return render_template("login.html")
    # POST requests here (implied by not being HTTP GET)
    username = request.form.get("username")
    password = request.form.get("password")
    remember = True if request.form.get("remember") else False

    user = User.query.filter_by(username=username).first()

    if not user or not check_password_hash(user.password, password):
        flash("Incorrect credentials!")
        return redirect(url_for("auth.login"))

    login_user(user, remember=remember)

    return redirect(url_for('controlpanel.controlpanel'))


@auth_blueprint.route("/logout")
@login_required
def logout():
    logout_user()
    return redirect(url_for("controlpanel.index"))
